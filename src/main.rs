use {
    clap::{App, Arg},
    log::{debug, info},
    reqwest,
    scraper::{Html, Selector},
    serde::Deserialize,
};

#[derive(Deserialize, Debug)]
struct DetailResponse {
    availabilities: Vec<AvailabilityResponse>,
    search_result: CenterResponse,
}

#[derive(Deserialize, Debug)]
struct AvailabilityResponse {
    date: String,
    slots: Vec<SlotResponse>,
}

#[derive(Deserialize, Debug)]
struct SlotResponse {
    agenda_id: u32,
    start_date: String,
    end_date: String,
}

#[derive(Deserialize, Debug)]
struct CenterResponse {
    address: String,
    city: String,
    name_with_title: String,
    zipcode: String,
    url: String,
}

fn get_center_id(element_id: &str) -> &str {
    element_id.rsplit('-').next().unwrap()
}

fn app() -> App<'static, 'static> {
    App::new("Doctoscrape")
        .version("0.1")
        .author("Jon C. <me@jonc.dev>")
        .about("Scrapes Doctolib for available appointments, prints out the matches")
        .arg(
            Arg::with_name("postal_code")
                .help("Postal code in which to perform the search")
                .required(true)
                .index(1)
                .value_name("POSTAL_CODE"),
        )
        .arg(
            Arg::with_name("city")
                .help("City name in which to perform the search")
                .short("c")
                .long("city")
                .takes_value(true)
                .default_value("paris")
                .value_name("CITY"),
        )
        .arg(
            Arg::with_name("exclude_postal_code")
                .help("Exclude centers at the given postal code")
                .short("x")
                .long("exclude")
                .takes_value(true)
                .value_name("POSTAL_CODE"),
        )
        .arg(
            Arg::with_name("pages")
                .help("Number of search results pages to scrape")
                .short("p")
                .long("pages")
                .default_value("1")
                .takes_value(true)
                .value_name("NUMBER_OF_PAGES"),
        )
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let matches = app().get_matches();
    let postal_code = matches.value_of("postal_code").unwrap();
    let city = matches.value_of("city").unwrap();
    let pages = matches.value_of("pages").unwrap().parse::<u32>().unwrap();
    let exclude_postal_code = matches.value_of("exclude_postal_code");

    for page in 0..pages {
        let search_url = if page == 0 {
            format!("https://www.doctolib.fr/vaccination-covid-19/{}-{}?ref_visit_motive_ids[]=6970&ref_visit_motive_ids[]=7005&force_max_limit=2", postal_code, city)
        } else {
            let real_page = page + 1;
            format!("https://www.doctolib.fr/vaccination-covid-19/{}-{}?ref_visit_motive_ids[]=6970&ref_visit_motive_ids[]=7005&force_max_limit=2&page={}", postal_code, city, real_page)
        };
        let resp = reqwest::get(search_url).await.unwrap();

        assert!(resp.status().is_success());

        let body = resp.text().await.unwrap();
        // parses string of HTML as a document
        let fragment = Html::parse_document(&body);
        // parses based on a CSS selector
        let results = Selector::parse(".dl-search-result").unwrap();

        // iterate over elements matching our selector
        for result in fragment.select(&results) {
            // get the center's id
            let id = get_center_id(result.value().id().unwrap());

            let details_response = reqwest::get(format!( "https://www.doctolib.fr/search_results/{}.json?limit=4&ref_visit_motive_ids[]=6970&ref_visit_motive_ids[]=7005&speciality_id=5494&search_result_format=json", id)).await.unwrap();

            debug!("{:?}", result.text().collect::<Vec<_>>().join(", "));
            let DetailResponse {
                search_result,
                availabilities,
                ..
            } = details_response.json().await.unwrap();
            if let Some(exclude_postal_code) = exclude_postal_code {
                if search_result.zipcode == exclude_postal_code {
                    continue;
                }
            }
            let mut times = vec![];
            for availability in availabilities {
                times.extend(
                    availability
                        .slots
                        .into_iter()
                        .map(|x| x.start_date)
                        .collect::<Vec<_>>(),
                );
            }
            if !times.is_empty() {
                let times = times.join("\n");
                let address = format!("{}, {}", search_result.address, search_result.zipcode);
                info!(
                    "{} at {} has slots!\nhttps://doctolib.fr{}\n{}",
                    search_result.name_with_title, address, search_result.url, times
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_details() {
        let data_string = r#"{
          "availabilities": [
            { "date": "2021-05-16", "slots": [] },
            { "date": "2021-05-17", "slots": [
                {
                  "agenda_id": 441940,
                  "end_date": "2021-05-17T09:12:00.000+02:00",
                  "practitioner_agenda_id": null,
                  "start_date": "2021-05-17T09:06:00.000+02:00",
                  "steps": [
                    {
                      "agenda_id": 441940,
                      "end_date": "2021-05-17T09:12:00.000+02:00",
                      "practitioner_agenda_id": null,
                      "start_date": "2021-05-17T09:06:00.000+02:00",
                      "visit_motive_id": 2700183
                    },
                    {
                      "agenda_id": 439320,
                      "end_date": "2021-06-25T08:36:00.000+02:00",
                      "practitioner_agenda_id": null,
                      "start_date": "2021-06-25T08:30:00.000+02:00",
                      "visit_motive_id": 2700184
                    }
                  ]
                }
              ]
            }
          ],
          "message": "Aucun rendez-vous n'est disponible pour le moment mais de nombreux créneaux vont être mis en ligne dans les jours à venir. Vous pouvez également prendre rendez-vous dans un autre lieu de vaccination.",
          "number_future_vaccinations": 1654,
          "reason": "no_availabilities",
          "search_result": {
            "address": "19b Place du Panthéon",
            "agenda_ids": [409357],
            "booking_temporary_disabled": false,
            "city": "Paris",
            "cloudinary_public_id": "uqpagkshgbmfxde5hi9w",
            "exact_match": null,
            "first_name": null,
            "id": 5965978,
            "is_directory": false,
            "landline_number": null,
            "last_name":
            "Centre COVID - Paris 5 ",
            "link": "/centre-de-sante/paris/centre-covid19-paris-5",
            "name_with_title": "Centre COVID - Paris 5",
            "organization_status": "Centre de santé",
            "place_id": null,
            "position": {
              "lat": 48.8457007,
              "lng": 2.344909
            },
            "priority_speciality": false,
            "profile_id": 188567,
            "resetVisitMotive": false,
            "speciality": null,
            "telehealth": false,
            "toFinalizeStep": false,
            "toFinalizeStepWithoutState": false,
            "top_specialities": ["1 salle de vaccination"],
            "url": "/centre-de-sante/paris/centre-covid19-paris-5?highlight[speciality_ids][]=5494",
            "visit_motive_id": 2860338,
            "visit_motive_name": "1re injection vaccin COVID-19 (Pfizer-BioNTech)",
            "zipcode": "75005"
          },
          "total": 0
        }"#;
        let details: DetailResponse = serde_json::from_str(data_string).unwrap();
        assert_eq!(details.search_result.zipcode, "75005");
        assert_eq!(details.search_result.address, "19b Place du Panthéon");
        assert_eq!(details.search_result.city, "Paris");
        assert_eq!(
            details.search_result.name_with_title,
            "Centre COVID - Paris 5"
        );
        assert_eq!(details.availabilities.len(), 2);
        let first_availability = &details.availabilities[0];
        assert_eq!(first_availability.date, "2021-05-16");
        assert!(first_availability.slots.is_empty());
        let second_availability = &details.availabilities[1];
        assert_eq!(second_availability.date, "2021-05-17");
        assert_eq!(second_availability.slots.len(), 1);
        assert_eq!(
            second_availability.slots[0].start_date,
            "2021-05-17T09:06:00.000+02:00"
        );
    }

    #[test]
    fn extract_id() {
        let element_id = "search-result-123";
        assert_eq!(get_center_id(&element_id), "123");
    }
}

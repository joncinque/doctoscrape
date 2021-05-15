use {
    reqwest,
    scraper::{node::Element, Html, Selector},
    serde::Deserialize,
    serde_json::Value,
};

const URL: &str = "https://www.doctolib.fr/vaccination-covid-19/75002-paris?ref_visit_motive_ids[]=6970&ref_visit_motive_ids[]=7005&force_max_limit=2";

#[derive(Deserialize, Debug)]
pub struct DetailResponse {
    availabilities: Vec<AvailabilityResponse>,
}

#[derive(Deserialize, Debug)]
pub struct AvailabilityResponse {
    date: String,
    slots: Vec<Value>, // TODO extract real data
}

fn get_center_id(element: &Element) -> &str {
    let id_string = element.id().unwrap();
    id_string.rsplit('-').next().unwrap()
}

#[tokio::main]
async fn main() {
    let resp = reqwest::get(URL).await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    // parses string of HTML as a document
    let fragment = Html::parse_document(&body);
    // parses based on a CSS selector
    let results = Selector::parse(".dl-search-result").unwrap();

    // iterate over elements matching our selector
    for result in fragment.select(&results) {
        let summary = result.text().collect::<Vec<_>>().join(", ");

        // get the center's id
        let id = get_center_id(result.value());
        println!("{:?}", summary);
        println!("{:?}", id);

        let details_response = reqwest::get(format!( "https://www.doctolib.fr/search_results/{}.json?limit=4&ref_visit_motive_ids%5B%5D=6970&ref_visit_motive_ids%5B%5D=7005&speciality_id=5494&search_result_format=json", id)).await.unwrap();

        let details = details_response.json::<DetailResponse>().await.unwrap();
        println!("{:?}", details.availabilities);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_details() {
        let data_string = r#"{
          "availabilities": [],
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
            "url": "/centre-de-sante/paris/centre-covid19-paris-5?highlight%5Bspeciality_ids%5D%5B%5D=5494",
            "visit_motive_id": 2860338,
            "visit_motive_name": "1re injection vaccin COVID-19 (Pfizer-BioNTech)",
            "zipcode": "75005"
          },
          "total": 0
        }"#;
        let details: DetailResponse = serde_json::from_str(data_string).unwrap();
        assert!(details.availabilities.is_empty());
    }
}

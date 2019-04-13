// Implements websequence diagram client
// Uses its API: http://www.websequencediagrams.com/embedding.html

use std::borrow::ToOwned;

use serde::{Serialize, Deserialize};
use serde_json::Deserializer;

use std::error::Error;

fn normalise_str(s: &str) -> String {
    s.to_lowercase()
        .replace("-", "")
        .replace("_", "")
}

pub trait WSDEnum where Self: std::marker::Sized {
    fn premium_feature(&self) -> bool;
    fn wsd_value(&self) -> String;
    fn all() -> Vec<Self>;

    fn from_str(s: &str) -> Option<Self> {
        for x in Self::all() {
            if normalise_str(&x.wsd_value()) == normalise_str(s) {
                return Some(x)
            }
        }
        None
    }

    fn all_wsd_values() -> Vec<String> {
        Self::all()
            .iter()
            .map(|x| x.wsd_value())
            .collect()
    }
}

// represent output format. Note: some formats (Pdf, Svg) are premium features
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Format {
    Png,
    Pdf,
    Svg,
    // TODO(mkl): High-res PNG ?
}

// By default PNG image is created
impl Default for Format {
    fn default() -> Format {
        Format::Png
    }
}

impl WSDEnum for Format {
    fn premium_feature(&self) -> bool {
        match self {
            Format::Png => false,
            Format::Pdf => true,
            Format::Svg => true
        }
    }

    // Converts to value used in WSD API
    fn wsd_value(&self) -> String {
        match self {
            Format::Png => "png".to_owned(),
            Format::Pdf => "pdf".to_owned(),
            Format::Svg => "svg".to_owned()
        }
    }

    fn all() -> Vec<Format> {
        use Format::*;
        vec![Png, Pdf, Svg]
    }
}

//default
//earth
//magazine
//modern-blue
//mscgen
//napkin
//omegapple
//patent
//qsd
//rose
//roundgreen
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Style {
    Default,
    Earth,
    Magazine,
    ModernBlue,
    Mscgen,
    Napkin,
    Omegapple,
    Patent,
    Qsd,
    Rose,
    Roundgreen
}

impl Default for Style {
    fn default() -> Style {
        Style::Default
    }
}

impl WSDEnum for Style {
    fn premium_feature(&self) -> bool {
        return false;
    }

    // Converts to value used in WSD API
    fn wsd_value(&self) -> String {
        match self {
            Style::Default => "default".to_owned(),
            Style::Earth => "earth".to_owned(),
            Style::Magazine => "magazine".to_owned(),
            Style::ModernBlue => "modern-blue".to_owned(),
            Style::Mscgen => "mscgen".to_owned(),
            Style::Napkin => "napkin".to_owned(),
            Style::Omegapple => "omegapple".to_owned(),
            Style::Patent => "patent".to_owned(),
            Style::Qsd => "qsd".to_owned(),
            Style::Rose => "rose".to_owned(),
            Style::Roundgreen => "roundgreen".to_owned()
        }
    }

    fn all() -> Vec<Style> {
        use Style::*;
        vec![Default, Earth, Magazine, ModernBlue, Mscgen, Napkin, Omegapple, Patent, Qsd, Rose, Roundgreen]
    }
}

// Represent response from websequence diagram website
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSequenceDiagramResponse {
    img: String,
    errors: Vec<String>

    // TODO(mkl): add aditional fields
}


pub fn get_diagram(spec: &str, style: &Style, format: &Format, api_key: Option<String>) -> Result<Vec<u8>, Box<Error>> {
    let resp = reqwest::Client::new()
        .post("http://www.websequencediagrams.com/index.php")
        .form(&[
            ("message", spec),
            ("style", &style.wsd_value()),
            ("format", &format.wsd_value()),
            ("apiVersion", "1")
        ])
        .send();
    let wr: WebSequenceDiagramResponse = match resp {
        Ok(mut r) => {
            match serde_json::from_reader(r) {
                Ok(r) => r,
                Err(err) => {
                    return Err(format!("Error deserializing websequencegiagram response: {:?}", err).into());
                }
            }
        },
        Err(err) =>  {
            return Err(format!("ERROR: {}", err).into());
        }
    };

    let mut resp2 = reqwest::Client::new()
        .get(("http://www.websequencediagrams.com/index.php".to_owned() + &wr.img).as_str())
        .send().unwrap();

    if !resp2.status().is_success() {
        return Err(format!("Error getting image from size").into())
    }

    let mut data = vec![];
    // copy the response body directly to stdout
    std::io::copy(&mut resp2, &mut data);
    Ok(data)
}

//#[cfg(test)]
//mod tests {
//
//    #[test]
//    fn test_normalise_str() {
//         asserteq!(1, 2);
//    }
//
//}
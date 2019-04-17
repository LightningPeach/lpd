// Implements websequence diagram client
// Uses its API: http://www.websequencediagrams.com/embedding.html

use std::borrow::ToOwned;

use serde::{Serialize, Deserialize};

use std::error::Error;

fn normalise_str(s: &str) -> String {
    s.to_lowercase().replace("-", "").replace("_", "")
}

pub trait WSDEnum
where
    Self: std::marker::Sized,
{
    fn premium_feature(&self) -> bool;
    fn wsd_value(&self) -> String;
    fn all() -> Vec<Self>;

    fn human_readable_value(&self) -> String {
        self.wsd_value()
    }

    fn from_str(s: &str) -> Option<Self> {
        for x in Self::all() {
            if normalise_str(&x.human_readable_value()) == normalise_str(s) {
                return Some(x);
            }
        }
        None
    }

    fn all_wsd_values() -> Vec<String> {
        Self::all().iter().map(WSDEnum::wsd_value).collect()
    }

    fn all_human_readable_values() -> Vec<String> {
        Self::all()
            .iter()
            .map(WSDEnum::human_readable_value)
            .collect()
    }

    fn help_str() -> String {
        Self::all()
            .iter()
            .map(|x| {
                if x.premium_feature() {
                    format!("{} (premium)", x.human_readable_value())
                } else {
                    x.human_readable_value()
                }
            })
            .collect::<Vec<String>>()
            .join(", ")
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
            Format::Svg => true,
        }
    }

    // Converts to value used in WSD API
    fn wsd_value(&self) -> String {
        match self {
            Format::Png => "png".to_owned(),
            Format::Pdf => "pdf".to_owned(),
            Format::Svg => "svg".to_owned(),
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
    Roundgreen,
}

impl Default for Style {
    fn default() -> Style {
        Style::Default
    }
}

impl WSDEnum for Style {
    fn premium_feature(&self) -> bool {
        false
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
            Style::Roundgreen => "roundgreen".to_owned(),
        }
    }

    fn all() -> Vec<Style> {
        use Style::*;
        vec![
            Default, Earth, Magazine, ModernBlue, Mscgen, Napkin, Omegapple, Patent, Qsd, Rose,
            Roundgreen,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaperSize {
    None,
    Letter,
    A4,
    _11x17,
    A1,
    A2,
    A3,
    Legal,
}

impl Default for PaperSize {
    fn default() -> PaperSize {
        PaperSize::None
    }
}

impl WSDEnum for PaperSize {
    fn premium_feature(&self) -> bool {
        // TODO(mkl): check if it is actually correct
        false
    }

    fn wsd_value(&self) -> String {
        match self {
            PaperSize::None => "none".to_owned(),
            PaperSize::Letter => "lettter".to_owned(),
            PaperSize::A4 => "a4".to_owned(),
            PaperSize::_11x17 => "11x17".to_owned(),
            PaperSize::A1 => "a1".to_owned(),
            PaperSize::A2 => "a2".to_owned(),
            PaperSize::A3 => "a3".to_owned(),
            PaperSize::Legal => "legal".to_owned(),
        }
    }

    fn all() -> Vec<PaperSize> {
        use PaperSize::*;
        vec![None, Letter, A4, _11x17, A1, A2, A3, Legal]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaperOrientation {
    Portrait,
    Landscape,
}

impl Default for PaperOrientation {
    fn default() -> PaperOrientation {
        PaperOrientation::Portrait
    }
}

impl WSDEnum for PaperOrientation {
    fn premium_feature(&self) -> bool {
        // TODO(mkl): check this
        false
    }

    // in request Portrait is encoded as landscape=0
    // Landscape is encoded as landscape=1
    fn wsd_value(&self) -> String {
        match self {
            PaperOrientation::Portrait => "0".to_owned(),
            PaperOrientation::Landscape => "1".to_owned(),
        }
    }

    fn all() -> Vec<PaperOrientation> {
        vec![PaperOrientation::Portrait, PaperOrientation::Landscape]
    }

    fn human_readable_value(&self) -> String {
        match self {
            PaperOrientation::Portrait => "portrait".to_owned(),
            PaperOrientation::Landscape => "landscape".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlotParameters {
    pub style: Style,
    pub format: Format,
    pub paper_size: Option<PaperSize>,
    pub paper_orientation: Option<PaperOrientation>,
    pub scale: Option<u32>,
    pub api_key: Option<String>,
}

impl Default for PlotParameters {
    fn default() -> PlotParameters {
        PlotParameters {
            style: Style::default(),
            format: Format::default(),
            paper_size: None,
            paper_orientation: None,
            scale: None,
            api_key: None,
        }
    }
}

// Represent response from websequence diagram website
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSequenceDiagramResponse {
    img: String,
    errors: Vec<String>, // TODO(mkl): add aditional fields
}

pub fn get_diagram(spec: &str, parameters: &PlotParameters) -> Result<Vec<u8>, Box<Error>> {
    // TODO(mkl): correct handling of incorrect API key
    // if send request for pdf but key is incorrect png in returned
    let mut params = vec![
        ("message".to_owned(), spec.to_owned()),
        ("style".to_owned(), parameters.style.wsd_value()),
        ("format".to_owned(), parameters.format.wsd_value()),
        ("apiVersion".to_owned(), "1".to_owned()),
    ];
    if let Some(ref api_key) = parameters.api_key {
        params.push(("apikey".to_owned(), api_key.clone()));
    }
    if let Some(ref paper_size) = parameters.paper_size {
        params.push(("paper".to_owned(), paper_size.wsd_value()));
    }
    if let Some(ref paper_orientation) = parameters.paper_orientation {
        params.push(("landscape".to_owned(), paper_orientation.wsd_value()));
    }
    if let Some(ref scale) = parameters.scale {
        params.push(("scale".to_owned(), format!("{}", scale)));
    }

    let resp = reqwest::Client::new()
        .post("http://www.websequencediagrams.com/index.php")
        .form(&params)
        .send();

    let wr: WebSequenceDiagramResponse = match resp {
        Ok(mut r) => {
            let mut v = vec![];
            // Save the response, so we can check it if something going wrong
            std::io::copy(&mut r, &mut v).unwrap();

            if !r.status().is_success() {
                return Err(format!(
                    "Error response from wsd code={:?} response={}",
                    r.status(),
                    String::from_utf8_lossy(&v)
                )
                .into());
            }

            println!("response: {}", String::from_utf8_lossy(&v));
            match serde_json::from_reader(&v[..]) {
                Ok(r) => r,
                Err(err) => {
                    println!(
                        "Error deserializing websequencegiagram response: {:?}",
                        &err
                    );
                    println!("Response: {}", String::from_utf8_lossy(&v));
                    return Err(format!(
                        "Error deserializing websequencegiagram response: {:?}",
                        err
                    )
                    .into());
                }
            }
        }
        Err(err) => {
            return Err(format!("ERROR: {}", err).into());
        }
    };

    let mut resp2 = reqwest::Client::new()
        .get(("http://www.websequencediagrams.com/index.php".to_owned() + &wr.img).as_str())
        .send()
        .unwrap();

    if !resp2.status().is_success() {
        return Err("Error getting image from size".to_string().into());
    }

    let mut data = vec![];
    // copy the response body directly to stdout
    std::io::copy(&mut resp2, &mut data).unwrap();
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

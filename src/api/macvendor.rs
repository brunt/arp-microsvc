//for calling macvendor api
//GET https://api.macvendors.com/98-01-A7 plaintext response

use reqwest::{get, Error as RWError};

//dashes are not needed for calling macvendors api, capitalization doesn't matter
pub fn vendor_request(url: &str, mac_addr: &str) -> Result<String, RWError> {
    let resp = get(&format!("{}/{}", url, mac_addr))?.text()?;
    Ok(resp)
}

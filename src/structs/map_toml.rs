use serde::{Deserialize, Serialize};

// This would be used to read the map TOML and convert it back to an internal LcfMapUnit struct.
// ...but why? What use is there? The only TOML you need to read are the patch files. Anything else is wasted effort for v1.0.
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfMapUnitToml {
    /*
    header: //...
    event: //...
    */
}

impl LcfMapUnitToml {
    /*pub fn convert_to_lcfmapunit(&self) {
        //
    }*/
}

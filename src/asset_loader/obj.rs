use std::path::{Path, PathBuf};
use super::tobj;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Obj or Mtl load error")]
    LoadError(#[source] tobj::LoadError),
    #[error("Obj path {path:?} must be absolute")]
    ObjPathMustBeAbsolute { path: PathBuf },
}

impl From<tobj::LoadError> for Error {
    fn from(other: tobj::LoadError) -> Self {
        Error::LoadError(other)
    }
}

pub struct ModelsWithMaterials {
    pub models: Vec<tobj::Model>,
    pub materials: Vec<tobj::Material>,
}

impl ModelsWithMaterials {
    pub fn load(path: &Path) -> Result<ModelsWithMaterials, Error> {
        if !path.is_absolute() {
            return Err(Error::ObjPathMustBeAbsolute { path: path.into() });
        }

        let (models, materials) = tobj::load_obj(path)?;
        debug!("models: {:?}", models.len());
        for model in &models {
            debug!("name: {:?}", model.name);
            debug!("positions {:?}", model.mesh.positions);
            debug!("normals {:?}", model.mesh.normals);
            debug!("texcoords {:?}", model.mesh.texcoords);
            debug!("indices {:?}", model.mesh.indices);
            debug!("material_id {:?}", model.mesh.material_id);
        }

        debug!("materials: {:?}", materials.len());
        for obj in &materials {
            debug!("material: {:#?}", obj);
        }

        Ok(ModelsWithMaterials { models, materials })
    }
}

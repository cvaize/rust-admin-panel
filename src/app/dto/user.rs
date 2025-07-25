use mime::{Mime, IMAGE_JPEG, IMAGE_PNG};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, VariantNames};

// 3 Megabytes
pub const USER_AVATAR_MAX_SIZE: usize = 3145728;
pub const USER_AVATAR_MIMES: &'static[Mime; 2] = &[IMAGE_JPEG, IMAGE_PNG];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub locale: Option<String>,
    pub surname: Option<String>,
    pub name: Option<String>,
    pub patronymic: Option<String>,
    pub is_super_admin: bool,
    pub roles_ids: Option<Vec<u64>>,
    pub avatar_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, Display, EnumString, Serialize, Deserialize, VariantNames, EnumIter, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum UserColumn {
    Id,
    Email,
    Locale,
    Surname,
    Name,
    Patronymic,
    IsSuperAdmin,
    RolesIds,
    AvatarId,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct UserCredentials {
    pub id: u64,
    pub email: String,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Copy, Display, EnumString, Serialize, Deserialize, VariantNames, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum UserCredentialsColumn {
    Id,
    Email,
    Password,
}

impl User {
    pub fn get_full_name_with_id_and_email(&self) -> String {
        let mut full_name = "".to_string();

        if let Some(surname) = &self.surname {
            full_name.push_str(surname);
        }

        if let Some(name) = &self.name {
            if full_name.len() != 0 {
                full_name.push(' ');
            }
            full_name.push_str(name);
        }

        if let Some(patronymic) = &self.patronymic {
            if full_name.len() != 0 {
                full_name.push(' ');
            }
            full_name.push_str(patronymic);
        }

        if full_name.len() != 0 {
            full_name.push_str(" - ");
        }

        full_name.push_str(&self.email);

        full_name.push_str("(ID:");
        full_name.push_str(&self.id.to_string());
        full_name.push(')');

        full_name
    }
    pub fn empty(email: String) -> Self {
        let mut entity = Self::default();
        entity.email = email;
        entity
    }
}

//! アプリケーション設定の取得関係

mod impl_neko_todo_config;
#[cfg(test)]
mod test;

use uuid::Uuid;

const CONF_FILE_NAME: &str = "neko_todo.conf";
const DB_HOST: &str = "NEKO_DB_DB_HOST";
const DB_USER: &str = "NEKO_DB_DB_USER";
const DB_PASS: &str = "NEKO_DB_DB_PASS";
const SESSION: &str = "NEKO_DB_SESSION_ID";

/// アプリケーション全体の状態設定
#[derive(Debug)]
pub struct NekoTodoConfig {
    db_host: String,
    db_user: String,
    db_pass: String,
    session_id: Option<Uuid>,
    dirty: bool,
    is_incomplete: bool,
    item_sort_order: ItemSortOrder,
}

/// アイテムリストのソート順位を表す。
#[derive(Debug, Clone, Copy)]
pub enum ItemSortOrder {
    StartAsc,
    StartDesc,
    EndAsc,
    EndDesc,
    UpdateAsc,
    UpdateDesc,
}

impl std::fmt::Display for ItemSortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartAsc => write!(f, "StartAsc"),
            Self::StartDesc => write!(f, "StartDesc"),
            Self::EndAsc => write!(f, "EndAsc"),
            Self::EndDesc => write!(f, "EndDesc"),
            Self::UpdateAsc => write!(f, "UpdateAsc"),
            Self::UpdateDesc => write!(f, "UpdateDesc"),
        }
    }
}

impl std::str::FromStr for ItemSortOrder {
    type Err = ItemSortOrderParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "StartAsc" => Ok(Self::StartAsc),
            "StartDesc" => Ok(Self::StartDesc),
            "EndAsc" => Ok(Self::EndAsc),
            "EndDesc" => Ok(Self::EndDesc),
            "UpdateAsc" => Ok(Self::UpdateAsc),
            "UpdateDesc" => Ok(Self::UpdateDesc),
            _ => Err(ItemSortOrderParseError::InvalidArgument),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ItemSortOrderParseError {
    #[error("Invalid Argument")]
    InvalidArgument,
}

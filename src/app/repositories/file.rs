use crate::helpers::{join_vec, now_date_time_str};
use crate::{take_from_mysql_row, take_some_datetime_from_mysql_row, AppError, Disk, File, FileColumn, FromMysqlDto, MysqlColumnEnum, MysqlIdColumn, MysqlPool, MysqlQueryBuilder, MysqlRepository, PaginateParams, Role, RoleFilter, ToMysqlDto};
use actix_web::web::Data;
use mysql::Value;
use mysql::Row;
use strum_macros::{Display, EnumIter, EnumString};

pub struct FileMysqlRepository {
    db_pool: Data<MysqlPool>,
}

impl MysqlRepository<File, FilePaginateParams, FileColumn, FileFilter, FileSort>
    for FileMysqlRepository
{
    fn get_repository_name(&self) -> &str {
        "FileMysqlRepository"
    }
    fn get_table(&self) -> &str {
        "files"
    }
    fn get_db_pool(&self) -> &MysqlPool {
        self.db_pool.get_ref()
    }
}

impl FileMysqlRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        Self { db_pool }
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<File>, AppError> {
        let filters = vec![FileFilter::Id(id)];
        self.first(&filters)
    }

    pub fn first_by_disk_and_path(
        &self,
        disk: &Disk,
        path: &str,
    ) -> Result<Option<File>, AppError> {
        let filters: Vec<FileFilter> = vec![
            FileFilter::Disk(disk.to_string()),
            FileFilter::Path(path.to_string()),
        ];
        self.first(&filters)
    }

    pub fn exists_by_disk_and_path(&self, disk: &Disk, path: &str) -> Result<bool, AppError> {
        let filters: Vec<FileFilter> = vec![
            FileFilter::Disk(disk.to_string()),
            FileFilter::Path(path.to_string()),
        ];
        self.exists(&filters)
    }

    pub fn delete_by_disk_and_path(&self, disk: &Disk, path: &str) -> Result<(), AppError> {
        let filters: Vec<FileFilter> = vec![
            FileFilter::Disk(disk.to_string()),
            FileFilter::Path(path.to_string()),
        ];
        self.delete(&filters)
    }

    pub fn first_by_disk_and_filename(
        &self,
        disk: &Disk,
        filename: &str,
    ) -> Result<Option<File>, AppError> {
        let filters: Vec<FileFilter> = vec![
            FileFilter::Filename(filename.to_string()),
            FileFilter::Disk(disk.to_string()),
        ];
        self.first(&filters)
    }

    pub fn soft_delete_by_id(&self, id: u64) -> Result<(), AppError> {
        let filters = vec![
            FileFilter::Id(id),
            FileFilter::IsDeleted(false),
        ];

        let mut data = File::default();
        data.delete_at = Some(now_date_time_str());
        data.is_delete = true;

        let columns: Option<Vec<FileColumn>> = Some(vec![
            FileColumn::DeleteAt,
            FileColumn::IsDelete,
        ]);

        self.update(&filters, &data, &columns)
    }

    pub fn soft_delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), AppError> {
        let filters = vec![
            FileFilter::Ids(ids.clone()),
            FileFilter::IsDeleted(false),
        ];

        let mut data = File::default();
        data.delete_at = Some(now_date_time_str());
        data.is_delete = true;

        let columns: Option<Vec<FileColumn>> = Some(vec![
            FileColumn::DeleteAt,
            FileColumn::IsDelete,
        ]);

        self.update(&filters, &data, &columns)
    }

    pub fn restore_by_id(&self, id: u64) -> Result<(), AppError> {
        let filters = vec![
            FileFilter::Id(id),
            FileFilter::IsDelete(true),
            FileFilter::IsDeleted(false),
        ];

        let mut data = File::default();
        data.delete_at = None;
        data.is_delete = false;

        let columns: Option<Vec<FileColumn>> = Some(vec![
            FileColumn::DeleteAt,
            FileColumn::IsDelete,
        ]);

        self.update(&filters, &data, &columns)
    }

    pub fn restore_by_ids(&self, ids: &Vec<u64>) -> Result<(), AppError> {
        let filters = vec![
            FileFilter::Ids(ids.clone()),
            FileFilter::IsDelete(true),
            FileFilter::IsDeleted(false),
        ];

        let mut data = File::default();
        data.delete_at = None;
        data.is_delete = false;

        let columns: Option<Vec<FileColumn>> = Some(vec![
            FileColumn::DeleteAt,
            FileColumn::IsDelete,
        ]);

        self.update(&filters, &data, &columns)
    }
}

pub type FilePaginateParams = PaginateParams<FileFilter, FileSort>;

#[derive(Debug)]
pub enum FileFilter {
    Id(u64),
    Ids(Vec<u64>),
    CreatorUserId(u64),
    Disk(String),
    Path(String),
    Filename(String),
    Search(String),
    IsDelete(bool),
    IsDeleted(bool),
}

impl MysqlQueryBuilder for FileFilter {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Id(_) => query.push_str("id=:f_id"),
            Self::Ids(value) => {
                let v = format!("id in ({})", join_vec(value, ","));
                query.push_str(&v)
            },
            Self::CreatorUserId(_) => query.push_str("creator_user_id=:f_creator_user_id"),
            Self::Disk(_) => query.push_str("disk=:f_disk"),
            Self::Path(_) => query.push_str("path=:f_path"),
            Self::Filename(_) => query.push_str("filename=:f_filename"),
            Self::Search(_) => query.push_str("(filename LIKE :f_search OR path LIKE :f_search)"),
            Self::IsDelete(_) => query.push_str("is_delete=:f_is_delete"),
            Self::IsDeleted(_) => query.push_str("is_deleted=:f_is_deleted"),
        }
    }

    fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        match self {
            Self::Id(value) => {
                params.push(("f_id".to_string(), Value::from(value)));
            }
            Self::Ids(_) => {}
            Self::CreatorUserId(value) => {
                params.push(("f_creator_user_id".to_string(), Value::from(value)));
            }
            Self::Disk(value) => {
                params.push(("f_disk".to_string(), Value::from(value)));
            }
            Self::Path(value) => {
                params.push(("f_path".to_string(), Value::from(value)));
            }
            Self::Filename(value) => {
                params.push(("f_filename".to_string(), Value::from(value)));
            }
            Self::Search(value) => {
                let mut s = "%".to_string();
                s.push_str(value);
                s.push_str("%");
                params.push(("f_search".to_string(), Value::from(s)));
            }
            Self::IsDelete(value) => {
                params.push(("f_is_delete".to_string(), Value::from(value)));
            }
            Self::IsDeleted(value) => {
                params.push(("f_is_deleted".to_string(), Value::from(value)));
            }
        }
    }
}

#[derive(Debug, Display, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum FileSort {
    IdAsc,
    IdDesc,
}

impl MysqlQueryBuilder for FileSort {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::IdAsc => query.push_str("id ASC"),
            Self::IdDesc => query.push_str("id DESC"),
        };
    }

    fn push_params_to_vec(&self, _: &mut Vec<(String, Value)>) {}
}

impl ToMysqlDto<FileColumn> for File {
    fn push_mysql_param_to_vec(&self, column: &FileColumn, params: &mut Vec<(String, Value)>) {
        match column {
            FileColumn::Id => params.push((column.to_string(), Value::from(self.id.to_owned()))),
            FileColumn::Filename => {
                params.push((column.to_string(), Value::from(self.filename.to_owned())))
            }
            FileColumn::Path => {
                params.push((column.to_string(), Value::from(self.path.to_owned())))
            }
            FileColumn::Mime => {
                params.push((column.to_string(), Value::from(self.mime.to_owned())))
            }
            FileColumn::Hash => {
                params.push((column.to_string(), Value::from(self.hash.to_owned())))
            }
            FileColumn::Size => {
                params.push((column.to_string(), Value::from(self.size.to_owned())))
            }
            FileColumn::CreatorUserId => params.push((
                column.to_string(),
                Value::from(self.creator_user_id.to_owned()),
            )),
            FileColumn::CreatedAt => {
                params.push((column.to_string(), Value::from(self.created_at.to_owned())))
            }
            FileColumn::UpdatedAt => {
                params.push((column.to_string(), Value::from(self.updated_at.to_owned())))
            }
            FileColumn::DeleteAt => {
                params.push((column.to_string(), Value::from(self.delete_at.to_owned())))
            }
            FileColumn::DeletedAt => {
                params.push((column.to_string(), Value::from(self.deleted_at.to_owned())))
            }
            FileColumn::IsDelete => {
                params.push((column.to_string(), Value::from(self.is_delete.to_owned())))
            }
            FileColumn::IsDeleted => {
                params.push((column.to_string(), Value::from(self.is_deleted.to_owned())))
            }
            FileColumn::Disk => {
                params.push((column.to_string(), Value::from(self.disk.to_owned())))
            }
        }
    }
    fn get_id(&self) -> u64 {
        self.id
    }
}

impl FromMysqlDto for File {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, AppError> {
        Ok(Self {
            id: take_from_mysql_row(row, FileColumn::Id.to_string().as_str())?,
            filename: take_from_mysql_row(row, FileColumn::Filename.to_string().as_str())?,
            path: take_from_mysql_row(row, FileColumn::Path.to_string().as_str())?,
            mime: take_from_mysql_row(row, FileColumn::Mime.to_string().as_str())?,
            hash: take_from_mysql_row(row, FileColumn::Hash.to_string().as_str())?,
            size: take_from_mysql_row(row, FileColumn::Size.to_string().as_str())?,
            creator_user_id: take_from_mysql_row(
                row,
                FileColumn::CreatorUserId.to_string().as_str(),
            )?,
            created_at: take_some_datetime_from_mysql_row(
                row,
                FileColumn::CreatedAt.to_string().as_str(),
            )?,
            updated_at: take_some_datetime_from_mysql_row(
                row,
                FileColumn::UpdatedAt.to_string().as_str(),
            )?,
            delete_at: take_some_datetime_from_mysql_row(
                row,
                FileColumn::DeleteAt.to_string().as_str(),
            )?,
            deleted_at: take_some_datetime_from_mysql_row(
                row,
                FileColumn::DeletedAt.to_string().as_str(),
            )?,
            is_delete: take_from_mysql_row(row, FileColumn::IsDelete.to_string().as_str())?,
            is_deleted: take_from_mysql_row(row, FileColumn::IsDeleted.to_string().as_str())?,
            disk: take_from_mysql_row(row, FileColumn::Disk.to_string().as_str())?,
            user_files: None,
        })
    }
}

impl MysqlColumnEnum for FileColumn {}
impl MysqlIdColumn for FileColumn {
    fn get_mysql_id_column() -> Self {
        Self::Id
    }
}

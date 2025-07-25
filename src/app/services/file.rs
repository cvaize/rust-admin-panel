#![allow(dead_code)]
use crate::helpers::now_date_time_str;
use crate::{AppError, Config, Disk, DiskExternalRepository, DiskLocalRepository, DiskRepository, File, FileColumn, FileFilter, FileMysqlRepository, FilePaginateParams, HashService, MysqlRepository, PaginationResult, RandomService, TranslatableError, TranslatorService, UserFile, UserFileColumn, UserFileFilter, UserFileMysqlRepository, UserFileService, UserFileServiceError, UserFileSort, UserServiceError};
use actix_web::web::Data;
use actix_web::{error, Error};
use mime::Mime;
use mime2ext::mime2ext;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::MAIN_SEPARATOR_STR;
use strum_macros::{Display, EnumString};

pub const FILE_DEFAULT_IS_PUBLIC: bool = false;
pub const FILE_DIRECTORY: &'static str = "files";

pub struct FileService {
    config: Data<Config>,
    file_repository: Data<FileMysqlRepository>,
    user_file_service: Data<UserFileService>,
    disk_local_repository: Data<DiskLocalRepository>,
    disk_external_repository: Data<DiskExternalRepository>,
    random_repository: Data<RandomService>,
    hash_service: Data<HashService>,
}

impl FileService {
    pub fn new(
        config: Data<Config>,
        file_repository: Data<FileMysqlRepository>,
        user_file_service: Data<UserFileService>,
        disk_local_repository: Data<DiskLocalRepository>,
        disk_external_repository: Data<DiskExternalRepository>,
        random_repository: Data<RandomService>,
        hash_service: Data<HashService>,
    ) -> Self {
        Self {
            config,
            file_repository,
            user_file_service,
            disk_local_repository,
            disk_external_repository,
            random_repository,
            hash_service,
        }
    }

    pub fn get_service_name(&self) -> &str {
        "FileService"
    }

    pub fn log_error(&self, method: &str, error: String, e: FileServiceError) -> FileServiceError {
        let service_name = self.get_service_name().to_string();
        log::error!("{}::{} - {}", service_name, method, error);
        e
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<File>, FileServiceError> {
        self.file_repository
            .get_ref()
            .first_by_id(id)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_id_throw_http(&self, id: u64) -> Result<File, Error> {
        let entity = self
            .first_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(entity) = entity {
            return Ok(entity);
        }
        Err(error::ErrorNotFound(""))
    }

    pub fn first_by_disk_and_filename(
        &self,
        disk: &Disk,
        filename: &str,
    ) -> Result<Option<File>, FileServiceError> {
        self.file_repository
            .get_ref()
            .first_by_disk_and_filename(disk, filename)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_disk_and_filename_throw_http(
        &self,
        disk: &Disk,
        filename: &str,
    ) -> Result<File, Error> {
        let entity = self
            .first_by_disk_and_filename(disk, filename)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(entity) = entity {
            return Ok(entity);
        }
        Err(error::ErrorNotFound(""))
    }

    fn match_error(&self, e: AppError) -> FileServiceError {
        let error = e.to_string();

        if error.contains("Duplicate entry") {
            if error.contains(".path") {
                return FileServiceError::DuplicateFile;
            }
            if error.contains(".file_id") {
                return FileServiceError::DuplicateFile;
            }
        }

        FileServiceError::Fail
    }

    fn match_user_service_error(&self, _: UserFileServiceError) -> FileServiceError {
        FileServiceError::UserFileServiceFail
    }

    pub fn create(&self, mut data: File) -> Result<(), FileServiceError> {
        if data.created_at.is_none() {
            data.created_at = Some(now_date_time_str());
        }
        if data.updated_at.is_none() {
            data.updated_at = Some(now_date_time_str());
        }
        let items = vec![data];
        self.file_repository
            .get_ref()
            .insert(&items, None)
            .map_err(|e| self.match_error(e))
    }

    pub fn update(
        &self,
        mut data: File,
        columns: &Option<Vec<FileColumn>>,
    ) -> Result<(), FileServiceError> {
        if data.created_at.is_none() {
            data.created_at = Some(now_date_time_str());
        }
        let filters = vec![FileFilter::Id(data.id)];
        data.updated_at = Some(now_date_time_str());
        self.file_repository
            .get_ref()
            .update(&filters, &data, columns)
            .map_err(|e| self.match_error(e))
    }

    pub fn upsert(
        &self,
        data: File,
        columns: &Option<Vec<FileColumn>>,
    ) -> Result<(), FileServiceError> {
        if data.id == 0 {
            self.create(data)
        } else {
            self.update(data, columns)
        }
    }

    pub fn soft_delete_by_id(&self, id: u64) -> Result<(), FileServiceError> {
        let _ = self
            .file_repository
            .get_ref()
            .soft_delete_by_id(id)
            .map_err(|e| self.match_error(e));
        self.user_file_service
            .get_ref()
            .soft_delete_by_file_id(id)
            .map_err(|e| self.match_user_service_error(e))
    }

    pub fn soft_delete_by_id_throw_http(&self, id: u64) -> Result<(), Error> {
        self.soft_delete_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn soft_delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), FileServiceError> {
        let _ = self
            .file_repository
            .get_ref()
            .soft_delete_by_ids(ids)
            .map_err(|e| self.match_error(e));
        self.user_file_service
            .get_ref()
            .soft_delete_by_file_ids(ids)
            .map_err(|e| self.match_user_service_error(e))
    }

    pub fn soft_delete_by_ids_throw_http(&self, ids: &Vec<u64>) -> Result<(), Error> {
        self.soft_delete_by_ids(ids)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn restore_by_id(&self, id: u64) -> Result<(), FileServiceError> {
        self.file_repository
            .get_ref()
            .restore_by_id(id)
            .map_err(|e| self.match_error(e))
    }

    pub fn restore_by_id_throw_http(&self, id: u64) -> Result<(), Error> {
        self.restore_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn restore_by_ids(&self, ids: &Vec<u64>) -> Result<(), FileServiceError> {
        self.file_repository
            .get_ref()
            .restore_by_ids(ids)
            .map_err(|e| self.match_error(e))
    }

    pub fn restore_by_ids_throw_http(&self, ids: &Vec<u64>) -> Result<(), Error> {
        self.restore_by_ids(ids)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn paginate(
        &self,
        params: &FilePaginateParams,
    ) -> Result<PaginationResult<File>, FileServiceError> {
        self.file_repository
            .get_ref()
            .paginate(params)
            .map_err(|e| self.match_error(e))
    }

    pub fn paginate_throw_http(
        &self,
        params: &FilePaginateParams,
    ) -> Result<PaginationResult<File>, Error> {
        self.paginate(params)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn upload_bytes_file_to_local_disk(
        &self,
        user_id: u64,
        bytes: Vec<u8>,
        is_public: bool,
        mut upload_filename: Option<String>,
        mut mime: Option<Mime>,
    ) -> Result<UserFile, FileServiceError> {
        let file_repository = self.file_repository.get_ref();
        let user_file_service = self.user_file_service.get_ref();
        let disk_local_repository = self.disk_local_repository.get_ref();
        let hash_service = self.hash_service.get_ref();

        if let Some(upload_filename_) = upload_filename {
            upload_filename = Some(upload_filename_.trim().to_string());
        }

        let disk = Disk::Local;

        // 1) Make filename [hash]-[size].[extensions]
        // 1.1) Make hash
        let hash: String = hash_service.hash(&bytes);

        if mime.is_none() {
            if let Some(upload_filename) = &upload_filename {
                mime = mime_guess::from_path(upload_filename).first()
            }
        }

        let mime_str: Option<String> = if let Some(mime) = &mime {
            Some(mime.to_string())
        } else {
            None
        };

        // 1.2) Make size
        let size: u64 = bytes.len() as u64;

        // 1.3) Make extensions
        // [hash]-[size].[extensions]
        let mut filename = hash.to_owned();
        filename.push('-');
        filename.push_str(size.to_string().as_str());

        if let Some(mime_) = &mime {
            if let Some(ext) = mime2ext(mime_) {
                filename.push('.');
                filename.push_str(ext.trim());
            }
        }

        // 2) Make path = [root]/[service_folder]/[filename]
        let path: String = disk_local_repository.path(&filename).map_err(|e| {
            self.log_error(
                "upload_bytes_file_to_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;

        // 3) Find old file in db or make new file
        let file: Option<File> = file_repository
            .first_by_disk_and_path(&disk, &path)
            .map_err(|e| {
                self.log_error(
                    "upload_bytes_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;

        let mut file: File = if let Some(file) = file {
            file
        } else {
            File::default()
        };

        // 4) Set new data in file
        let mut is_upsert = file.id == 0;

        if file.id == 0 {
            file.disk = disk.to_string();
        } else if file.disk.ne(disk.to_string().as_str()) {
            return Err(self.log_error(
                "upload_bytes_file_to_local_disk",
                format!("Disk not equal. File ID: {}.", file.id),
                FileServiceError::Fail,
            ));
        }

        if path.ne(&file.path) {
            file.path = path;
            is_upsert = true;
        }

        if filename.ne(&file.filename) {
            file.filename = filename;
            is_upsert = true;
        }

        let hash: Option<String> = Some(hash);
        if hash.ne(&file.hash) {
            file.hash = hash;
            is_upsert = true;
        }

        let size: Option<u64> = Some(size);
        if size.ne(&file.size) {
            file.size = size;
            is_upsert = true;
        }

        if mime_str.ne(&file.mime) {
            file.mime = mime_str;
            is_upsert = true;
        }

        if file.creator_user_id.is_none() {
            file.creator_user_id = Some(user_id);
            is_upsert = true;
        }

        if file.delete_at.is_some() {
            file.delete_at = None;
            is_upsert = true;
        }

        if file.deleted_at.is_some() {
            file.deleted_at = None;
            is_upsert = true;
        }

        if file.is_delete == true {
            file.is_delete = false;
            is_upsert = true;
        }

        if file.is_deleted == true {
            file.is_deleted = false;
            is_upsert = true;
        }

        // 5) Copy content if necessary.
        let is_exists_in_fs = disk_local_repository.exists(&file.path).map_err(|e| {
            self.log_error(
                "upload_bytes_file_to_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;

        let mut is_delete_old_file = false;
        let mut is_copy = !is_exists_in_fs;

        if is_exists_in_fs {
            if let Some(new_file_hash) = &file.hash {
                let old_file_hash: String =
                    disk_local_repository.hash(&file.path).map_err(|e| {
                        self.log_error(
                            "upload_bytes_file_to_local_disk",
                            e.to_string(),
                            FileServiceError::Fail,
                        )
                    })?;

                if old_file_hash.ne(new_file_hash) {
                    is_copy = true;
                    is_delete_old_file = true;
                }
            } else {
                is_copy = true;
                is_delete_old_file = true;
            }
        }

        if is_delete_old_file {
            disk_local_repository.delete(&file.path).map_err(|e| {
                self.log_error(
                    "upload_bytes_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;
        }

        if is_copy {
            disk_local_repository
                .put(&file.path, bytes)
                .map_err(|e| {
                    self.log_error(
                        "upload_bytes_file_to_local_disk",
                        e.to_string(),
                        FileServiceError::Fail,
                    )
                })?;
        }

        // 6) Upsert file meta in db
        if is_upsert {
            self.upsert(file.to_owned(), &None)?;

            let file_: Option<File> = file_repository
                .first_by_disk_and_path(&disk, &file.path)
                .map_err(|e| {
                    self.log_error(
                        "upload_bytes_file_to_local_disk",
                        e.to_string(),
                        FileServiceError::Fail,
                    )
                })?;
            if let Some(file_) = file_ {
                file = file_;
            } else {
                return Err(self.log_error(
                    "upload_bytes_file_to_local_disk",
                    format!("File created, but not found {}", &file.path),
                    FileServiceError::NotFound,
                ));
            }
        }

        // 7) Upsert user file in db
        let user_file: Option<UserFile> = user_file_service
            .first_by_user_id_and_file_id(user_id, file.id)
            .map_err(|e| {
                self.log_error(
                    "upload_bytes_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;

        let mut user_file: UserFile = if let Some(user_file) = user_file {
            user_file
        } else {
            UserFile::default()
        };

        // 8) Set new data in user file
        let mut is_upsert = user_file.id == 0;

        if user_file.id == 0 {
            user_file.user_id = user_id;
        } else if user_file.user_id.ne(&user_id) {
            return Err(self.log_error(
                "upload_bytes_file_to_local_disk",
                format!("User ID not equal. User File ID: {}.", user_file.id),
                FileServiceError::Fail,
            ));
        }

        if user_file.id == 0 {
            user_file.file_id = file.id;
        } else if user_file.file_id.ne(&file.id) {
            return Err(self.log_error(
                "upload_bytes_file_to_local_disk",
                format!("File ID not equal. User File ID: {}.", user_file.id),
                FileServiceError::Fail,
            ));
        }

        if upload_filename.ne(&user_file.upload_filename) {
            user_file.upload_filename = upload_filename;
            is_upsert = true;
        }

        if user_file.mime.ne(&file.mime) {
            user_file.mime = file.mime.to_owned();
            is_upsert = true;
        }

        if user_file.deleted_at.is_some() {
            user_file.deleted_at = None;
            is_upsert = true;
        }

        if user_file.is_deleted == true {
            user_file.is_deleted = false;
            is_upsert = true;
        }

        if user_file.disk.ne(&file.disk) {
            user_file.disk = file.disk.to_owned();
            is_upsert = true;
        }

        if user_file.is_public != is_public {
            user_file.is_public = is_public;
            is_upsert = true;
        }

        if is_upsert {
            let user_id = user_file.user_id;
            let file_id = user_file.file_id;
            user_file_service.upsert(user_file, &None, &file).map_err(|e| {
                self.log_error(
                    "upload_bytes_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;

            let user_file_: Option<UserFile> = user_file_service
                .first_by_user_id_and_file_id(user_id, file_id)
                .map_err(|e| {
                    self.log_error(
                        "upload_bytes_file_to_local_disk",
                        e.to_string(),
                        FileServiceError::Fail,
                    )
                })?;
            if let Some(user_file_) = user_file_ {
                user_file = user_file_;
            } else {
                return Err(self.log_error(
                    "upload_bytes_file_to_local_disk",
                    format!("User File created, but not found {}", &file.path),
                    FileServiceError::NotFound,
                ));
            }
        }

        Ok(user_file)
    }

    pub fn upload_local_file_to_local_disk(
        &self,
        user_id: u64,
        upload_path: &str,
        is_public: bool,
        mut upload_filename: Option<String>,
        mut mime: Option<Mime>,
    ) -> Result<UserFile, FileServiceError> {
        let file_repository = self.file_repository.get_ref();
        let user_file_service = self.user_file_service.get_ref();
        let disk_local_repository = self.disk_local_repository.get_ref();

        let is_exists = disk_local_repository.exists(upload_path).map_err(|e| {
            self.log_error(
                "upload_local_file_to_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;
        if !is_exists {
            return Err(self.log_error(
                "upload_local_file_to_local_disk",
                format!("File not found {}", upload_path),
                FileServiceError::NotFound,
            ));
        }

        if let Some(upload_filename_) = upload_filename {
            upload_filename = Some(upload_filename_.trim().to_string());
        }

        let disk = Disk::Local;

        // 1) Make filename [hash]-[size].[extensions]
        // 1.1) Make hash
        let hash: String = disk_local_repository.hash(upload_path).map_err(|e| {
            self.log_error(
                "upload_local_file_to_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;

        if mime.is_none() {
            mime = mime_guess::from_path(upload_path).first()
        }

        let mime_str: Option<String> = if let Some(mime) = &mime {
            Some(mime.to_string())
        } else {
            None
        };

        // 1.2) Make size
        let size = disk_local_repository.size(upload_path).map_err(|e| {
            self.log_error(
                "upload_local_file_to_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;

        // 1.3) Make extensions
        let mut mimes_from_path: Vec<Mime> = Vec::new();

        let upload_path_ = upload_path.to_lowercase();
        let last_path = upload_path_.split(MAIN_SEPARATOR_STR).last();
        let mut i = 0;
        if let Some(last_path) = last_path {
            for path_ in last_path.split('.') {
                i += 1;
                if i > 20 {
                    // A stub in case of an attack through multiple file extensions.
                    mimes_from_path = Vec::new();
                    break;
                }
                let g_mime: Option<Mime> = mime_guess::from_ext(path_.trim()).first();
                if let Some(mime_) = g_mime {
                    mimes_from_path.push(mime_);
                }
            }
        }

        if let Some(mime) = mime {
            if let Some(mime_) = mimes_from_path.last() {
                if mime.ne(mime_) {
                    mimes_from_path.push(mime.to_owned());
                }
            } else {
                mimes_from_path.push(mime.to_owned());
            }
        }
        let mut extensions: String = "".to_string();
        for mime_ in mimes_from_path {
            if let Some(ext) = mime2ext(mime_) {
                if !extensions.is_empty() {
                    extensions.push('.');
                }
                extensions.push_str(ext.trim());
            }
        }

        // [hash]-[size].[extensions]
        let mut filename = hash.to_owned();
        filename.push('-');
        filename.push_str(size.to_string().as_str());

        if !extensions.is_empty() {
            filename.push('.');
            filename.push_str(extensions.as_str());
        }

        // 2) Make path = [root]/[service_folder]/[filename]
        let path: String = disk_local_repository.path(&filename).map_err(|e| {
            self.log_error(
                "upload_local_file_to_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;

        // 3) Find old file in db or make new file
        let file: Option<File> = file_repository
            .first_by_disk_and_path(&disk, &path)
            .map_err(|e| {
                self.log_error(
                    "upload_local_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;

        let mut file: File = if let Some(file) = file {
            file
        } else {
            File::default()
        };

        // 4) Set new data in file
        let mut is_upsert = file.id == 0;

        if file.id == 0 {
            file.disk = disk.to_string();
        } else if file.disk.ne(disk.to_string().as_str()) {
            return Err(self.log_error(
                "upload_local_file_to_local_disk",
                format!("Disk not equal. File ID: {}.", file.id),
                FileServiceError::Fail,
            ));
        }

        if path.ne(&file.path) {
            file.path = path;
            is_upsert = true;
        }

        if filename.ne(&file.filename) {
            file.filename = filename;
            is_upsert = true;
        }

        let hash: Option<String> = Some(hash);
        if hash.ne(&file.hash) {
            file.hash = hash;
            is_upsert = true;
        }

        let size: Option<u64> = Some(size);
        if size.ne(&file.size) {
            file.size = size;
            is_upsert = true;
        }

        if mime_str.ne(&file.mime) {
            file.mime = mime_str;
            is_upsert = true;
        }

        if file.creator_user_id.is_none() {
            file.creator_user_id = Some(user_id);
            is_upsert = true;
        }

        if file.delete_at.is_some() {
            file.delete_at = None;
            is_upsert = true;
        }

        if file.deleted_at.is_some() {
            file.deleted_at = None;
            is_upsert = true;
        }

        if file.is_delete == true {
            file.is_delete = false;
            is_upsert = true;
        }

        if file.is_deleted == true {
            file.is_deleted = false;
            is_upsert = true;
        }

        // 5) Copy content if necessary.
        let is_exists_in_fs = disk_local_repository.exists(&file.path).map_err(|e| {
            self.log_error(
                "upload_local_file_to_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;

        let mut is_delete_old_file = false;
        let mut is_copy = !is_exists_in_fs;

        if is_exists_in_fs {
            if let Some(new_file_hash) = &file.hash {
                let old_file_hash: String =
                    disk_local_repository.hash(&file.path).map_err(|e| {
                        self.log_error(
                            "upload_local_file_to_local_disk",
                            e.to_string(),
                            FileServiceError::Fail,
                        )
                    })?;

                if old_file_hash.ne(new_file_hash) {
                    is_copy = true;
                    is_delete_old_file = true;
                }
            } else {
                is_copy = true;
                is_delete_old_file = true;
            }
        }

        if is_delete_old_file {
            disk_local_repository.delete(&file.path).map_err(|e| {
                self.log_error(
                    "upload_local_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;
        }

        if is_copy {
            let mut read_stream = disk_local_repository
                .read_stream(upload_path)
                .map_err(|e| {
                    self.log_error(
                        "upload_local_file_to_local_disk",
                        e.to_string(),
                        FileServiceError::Fail,
                    )
                })?;
            let mut write_stream = disk_local_repository
                .write_stream(&file.path)
                .map_err(|e| {
                    self.log_error(
                        "upload_local_file_to_local_disk",
                        e.to_string(),
                        FileServiceError::Fail,
                    )
                })?;
            let mut buf: Vec<u8> = Vec::new();
            while read_stream.read_to_end(&mut buf).map_err(|e| {
                self.log_error(
                    "upload_local_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })? > 0 {
                write_stream.write_all(&buf).map_err(|e| {
                    self.log_error(
                        "upload_local_file_to_local_disk",
                        e.to_string(),
                        FileServiceError::Fail,
                    )
                })?;
                buf.clear();
            }
            write_stream.flush().map_err(|e| {
                self.log_error(
                    "upload_local_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;
        }

        // 6) Upsert file meta in db
        if is_upsert {
            self.upsert(file.to_owned(), &None)?;

            let file_: Option<File> = file_repository
                .first_by_disk_and_path(&disk, &file.path)
                .map_err(|e| {
                    self.log_error(
                        "upload_local_file_to_local_disk",
                        e.to_string(),
                        FileServiceError::Fail,
                    )
                })?;
            if let Some(file_) = file_ {
                file = file_;
            } else {
                return Err(self.log_error(
                    "upload_local_file_to_local_disk",
                    format!("File created, but not found {}", &file.path),
                    FileServiceError::NotFound,
                ));
            }
        }

        // 7) Upsert user file in db
        let user_file: Option<UserFile> = user_file_service
            .first_by_user_id_and_file_id(user_id, file.id)
            .map_err(|e| {
                self.log_error(
                    "upload_local_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;

        let mut user_file: UserFile = if let Some(user_file) = user_file {
            user_file
        } else {
            UserFile::default()
        };

        // 8) Set new data in user file
        let mut is_upsert = user_file.id == 0;

        if user_file.id == 0 {
            user_file.user_id = user_id;
        } else if user_file.user_id.ne(&user_id) {
            return Err(self.log_error(
                "upload_local_file_to_local_disk",
                format!("User ID not equal. User File ID: {}.", user_file.id),
                FileServiceError::Fail,
            ));
        }

        if user_file.id == 0 {
            user_file.file_id = file.id;
        } else if user_file.file_id.ne(&file.id) {
            return Err(self.log_error(
                "upload_local_file_to_local_disk",
                format!("File ID not equal. User File ID: {}.", user_file.id),
                FileServiceError::Fail,
            ));
        }

        if upload_filename.ne(&user_file.upload_filename) {
            user_file.upload_filename = upload_filename;
            is_upsert = true;
        }

        if user_file.mime.ne(&file.mime) {
            user_file.mime = file.mime.to_owned();
            is_upsert = true;
        }

        if user_file.deleted_at.is_some() {
            user_file.deleted_at = None;
            is_upsert = true;
        }

        if user_file.is_deleted == true {
            user_file.is_deleted = false;
            is_upsert = true;
        }

        if user_file.disk.ne(&file.disk) {
            user_file.disk = file.disk.to_owned();
            is_upsert = true;
        }

        if user_file.is_public != is_public {
            user_file.is_public = is_public;
            is_upsert = true;
        }

        if is_upsert {
            let user_id = user_file.user_id;
            let file_id = user_file.file_id;
            user_file_service.upsert(user_file, &None, &file).map_err(|e| {
                self.log_error(
                    "upload_local_file_to_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;

            let user_file_: Option<UserFile> = user_file_service
                .first_by_user_id_and_file_id(user_id, file_id)
                .map_err(|e| {
                    self.log_error(
                        "upload_local_file_to_local_disk",
                        e.to_string(),
                        FileServiceError::Fail,
                    )
                })?;
            if let Some(user_file_) = user_file_ {
                user_file = user_file_;
            } else {
                return Err(self.log_error(
                    "upload_local_file_to_local_disk",
                    format!("User File created, but not found {}", &file.path),
                    FileServiceError::NotFound,
                ));
            }
        }

        Ok(user_file)
    }

    pub fn load_and_attach_user_files(
        &self,
        files: &mut Vec<File>,
        filters: Option<&Vec<UserFileFilter>>,
        sorts: Option<&Vec<UserFileSort>>,
    ) -> Result<(), FileServiceError> {
        if files.is_empty() {
            return Ok(());
        }

        let user_file_service = self.user_file_service.get_ref();

        let mut filters_: Vec<UserFileFilter> = vec![UserFileFilter::FileIds(
            files.iter().map(|f| f.id).collect(),
        )];

        if let Some(filters) = filters {
            for filter in filters {
                filters_.push(filter.to_owned());
            }
        }

        let user_files = user_file_service.all(Some(&filters_), sorts).map_err(|e| {
            self.log_error(
                "load_and_attach_user_files",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;
        let mut user_files_idx: HashMap<u64, Vec<UserFile>> = HashMap::new();

        for user_file in user_files {
            if let Some(user_files) = user_files_idx.get_mut(&user_file.file_id) {
                user_files.push(user_file);
            } else {
                user_files_idx.insert(user_file.file_id, vec![user_file]);
            }
        }

        for file in files {
            file.user_files = user_files_idx.remove(&file.id);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, PartialEq, Eq)]
pub enum FileServiceError {
    DbConnectionFail,
    DuplicateFile,
    NotFound,
    Fail,
    UserFileServiceFail,
}

impl TranslatableError for FileServiceError {
    fn translate(&self, lang: &str, translator_service: &TranslatorService) -> String {
        match self {
            Self::DbConnectionFail => {
                translator_service.translate(lang, "error.FileServiceError.DbConnectionFail")
            }
            Self::DuplicateFile => {
                translator_service.translate(lang, "error.FileServiceError.DuplicateFile")
            }
            Self::NotFound => translator_service.translate(lang, "error.FileServiceError.NotFound"),
            Self::UserFileServiceFail => {
                translator_service.translate(lang, "error.UserFileServiceError.Fail")
            }
            _ => translator_service.translate(lang, "error.FileServiceError.Fail"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{preparation, Disk, MysqlRepository};
    use mime::Mime;
    use std::path::MAIN_SEPARATOR_STR;
    use std::{env, fs};
    use test::Bencher;

    // #[test]
    // fn test_upload_local_file_to_local_disk() {
    //     // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::services::file::tests::test_upload_local_file_to_local_disk
    //     let (_, all_services) = preparation();
    //     let config = all_services.config.get_ref();
    //     let file_service = all_services.file_service.get_ref();
    //     let file_repository = all_services.file_mysql_repository.get_ref();
    //     let user_file_repository = all_services.user_file_mysql_repository.get_ref();
    //
    //     let root = env::current_dir().unwrap();
    //     let root_dir = root.to_str().unwrap();
    //
    //     let disk = Disk::Local;
    //     let user_id = 1;
    //     let is_public = true;
    //     let user_filename = "test_upload_local_file_to_local_disk.test.tar.gz";
    //     let hash = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".to_string();
    //
    //     let mut upload_path = root_dir.to_string();
    //     if !upload_path.ends_with(MAIN_SEPARATOR_STR) {
    //         upload_path.push_str(MAIN_SEPARATOR_STR);
    //     }
    //     upload_path.push_str(user_filename);
    //
    //     fs::write(&user_filename, "test").unwrap();
    //
    //     let mime: Mime = mime_guess::from_path(&upload_path).first().unwrap();
    //
    //     let user_file = file_service
    //         .upload_local_file_to_local_disk(
    //             user_id,
    //             &upload_path,
    //             is_public,
    //             Some(user_filename.to_owned()),
    //             Some(mime.to_owned()),
    //         )
    //         .unwrap();
    //
    //     let file = file_service
    //         .first_file_by_id(user_file.file_id)
    //         .unwrap()
    //         .unwrap();
    //
    //     // dbg!(&user_file);
    //     // dbg!(&file);
    //
    //     assert_eq!(user_file.user_id, user_id);
    //     assert_eq!(user_file.file_id, file.id);
    //
    //     let mut path = config.filesystem.disks.local.public_root.to_owned();
    //     if !path.ends_with(MAIN_SEPARATOR_STR) {
    //         path.push_str(MAIN_SEPARATOR_STR);
    //     }
    //     path.push_str(format!("1-{}-4.tar.gz", &hash).as_str());
    //     assert_eq!(user_file.path.as_str(), path.as_str());
    //     let m = Some(mime.to_string());
    //     assert_eq!(&user_file.mime, &m);
    //     assert_eq!(user_file.is_public, is_public);
    //     assert_eq!(file.filename.as_str(), format!("{}-4.tar.gz", &hash).as_str());
    //
    //     let mut path = config.filesystem.disks.local.root.to_owned();
    //     if !path.ends_with(MAIN_SEPARATOR_STR) {
    //         path.push_str(MAIN_SEPARATOR_STR);
    //     }
    //     path.push_str(format!("{}-4.tar.gz", &hash).as_str());
    //     assert_eq!(file.path.as_str(), path.as_str());
    //     let m = Some(mime.to_string());
    //     assert_eq!(&file.mime, &m);
    //     let s = Some(4);
    //     assert_eq!(&file.size, &s);
    //     assert_eq!(&file.disk, disk.to_string().as_str());
    //
    //     fs::remove_file(&user_filename).unwrap();
    //     fs::remove_file(&user_file.path).unwrap();
    //     fs::remove_file(&file.path).unwrap();
    //     user_file_repository.delete_by_id(user_file.id).unwrap();
    //     file_repository.delete_by_id(file.id).unwrap();
    // }

    // #[bench]
    // fn bench_encrypt_string(b: &mut Bencher) {
    //     let (_, all_services) = preparation();
    //     let crypt = all_services.crypt_service.get_ref();
    //
    //     b.iter(|| {
    //         let _ = crypt.encrypt_string(DATA).unwrap();
    //     })
    // }
}

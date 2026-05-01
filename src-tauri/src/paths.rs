//! 앱 데이터/캐시/스캔 결과 등 OS-별 디렉터리 해석 헬퍼.
//!
//! Rule 1: 결과 이미지는 임시 폴더에 저장 후 `asset://` 프로토콜로 노출하므로,
//! 모든 경로 결정은 이 모듈을 단일 출처로 사용한다.

use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::error::{AppError, AppResult};

/// SQLite 파일명 — `tauri-plugin-sql`의 `sqlite:` URL 과 일관되게 유지.
pub const DB_FILENAME: &str = "shalgalt-omr.sqlite";

/// 앱이 사용하는 표준 디렉터리 묶음.
#[derive(Debug, Clone)]
pub struct AppDirs {
    /// SQLite DB / 영속 설정.
    pub data_dir: PathBuf,
    /// PDF 분해 산출물 / 결과 이미지(asset:// 노출용).
    pub scans_dir: PathBuf,
    /// 일회성 임시 파일.
    pub cache_dir: PathBuf,
}

impl AppDirs {
    /// `directories` 크레이트가 OS 별로 적절한 위치를 결정한다.
    pub fn resolve() -> AppResult<Self> {
        let dirs = ProjectDirs::from("dev", "filename", "shalgalt-omr").ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "failed to resolve project directories for current OS"
            ))
        })?;

        let data_dir = dirs.data_dir().to_path_buf();
        let cache_dir = dirs.cache_dir().to_path_buf();
        let scans_dir = data_dir.join("scans");

        for d in [&data_dir, &cache_dir, &scans_dir] {
            ensure_dir(d)?;
        }

        Ok(Self {
            data_dir,
            scans_dir,
            cache_dir,
        })
    }

    /// SQLite 파일의 절대 경로.
    /// (참고용 — 실제 DB 접근은 `tauri-plugin-sql`이 AppConfig 디렉터리에서 수행한다.)
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join(DB_FILENAME)
    }
}

fn ensure_dir(p: &Path) -> AppResult<()> {
    if !p.exists() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}

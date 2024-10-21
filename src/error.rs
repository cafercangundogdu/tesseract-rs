use std::str::Utf8Error;
use thiserror::Error;

/// Tesseract API kullanırken oluşabilecek hatalar.
#[derive(Error, Debug)]
pub enum TesseractError {
    #[error("Tesseract başlatılamadı")]
    InitError,
    #[error("Görüntü ayarlanamadı")]
    SetImageError,
    #[error("OCR işlemi gerçekleştirilemedi")]
    OcrError,
    #[error("Tesseract çıktısında geçersiz UTF-8")]
    Utf8Error(#[from] Utf8Error),
    #[error("Mutex kilitleme hatası")]
    MutexLockError,
    #[error("Değişken ayarlanamadı")]
    SetVariableError,
    #[error("Değişken alınamadı")]
    GetVariableError,
    #[error("Null işaretçi hatası")]
    NullPointerError,
    #[error("Geçersiz parametre")]
    InvalidParameterError,
    #[error("Düzen analizi başarısız oldu")]
    AnalyseLayoutError,
    #[error("Sayfa işleme başarısız oldu")]
    ProcessPagesError,
    #[error("G/Ç hatası")]
    IoError,
    #[error("Mutex hatası")]
    MutexError,
}

/// Tesseract işlemleri için sonuç türü.
pub type Result<T> = std::result::Result<T, TesseractError>;

use crate::common::error::ServiceError;

pub fn validate_non_empty(field: &str, value: &str, max_len: usize) -> Result<(), ServiceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ServiceError::InvalidOperation(format!("{field}不能为空")));
    }
    if trimmed.len() > max_len {
        return Err(ServiceError::InvalidOperation(format!("{field}长度不能超过{max_len}个字符")));
    }
    Ok(())
}

pub fn validate_optional_text(
    field: &str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ServiceError> {
    if let Some(value) = value {
        if value.trim().is_empty() {
            return Err(ServiceError::InvalidOperation(format!("{field}不能为空字符串")));
        }
        if value.len() > max_len {
            return Err(ServiceError::InvalidOperation(format!(
                "{field}长度不能超过{max_len}个字符"
            )));
        }
    }
    Ok(())
}

pub fn validate_email(field: &str, value: &str, max_len: usize) -> Result<(), ServiceError> {
    validate_non_empty(field, value, max_len)?;
    let trimmed = value.trim();
    let mut parts = trimmed.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    let has_extra = parts.next().is_some();

    let valid = !local.is_empty()
        && !domain.is_empty()
        && !has_extra
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.');

    if !valid {
        return Err(ServiceError::InvalidOperation(format!("{field}格式不正确")));
    }
    Ok(())
}

pub fn validate_password(field: &str, value: &str) -> Result<(), ServiceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ServiceError::InvalidOperation(format!("{field}不能为空")));
    }
    if !(6..=128).contains(&value.len()) {
        return Err(ServiceError::InvalidOperation(format!("{field}长度必须在6到128个字符之间")));
    }
    Ok(())
}

pub fn validate_i16_in(field: &str, value: i16, allowed: &[i16]) -> Result<(), ServiceError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(ServiceError::InvalidOperation(format!("{field}取值非法")))
    }
}

pub fn validate_pagination(
    current: Option<i64>,
    page_size: Option<i64>,
) -> Result<(), ServiceError> {
    if let Some(current) = current {
        if current <= 0 {
            return Err(ServiceError::InvalidOperation("current必须大于0".into()));
        }
    }
    if let Some(page_size) = page_size {
        if !(1..=100).contains(&page_size) {
            return Err(ServiceError::InvalidOperation("pageSize必须在1到100之间".into()));
        }
    }
    Ok(())
}

pub fn validate_limit(limit: Option<i64>, max: i64) -> Result<(), ServiceError> {
    if let Some(limit) = limit {
        if !(1..=max).contains(&limit) {
            return Err(ServiceError::InvalidOperation(format!("limit必须在1到{max}之间")));
        }
    }
    Ok(())
}

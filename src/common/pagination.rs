/// 分页参数转换工具
pub struct Pagination;

impl Pagination {
    /// 标准化分页参数，返回 (page, limit, offset)
    pub fn normalize(current: Option<i64>, page_size: Option<i64>) -> (i64, i64, i64) {
        let page = current.unwrap_or(1).max(1);
        let limit = page_size.unwrap_or(10).min(100).max(1);
        let offset = (page - 1) * limit;
        (limit, offset, page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_none() {
        // page=1, limit=10, offset=0
        let (limit, offset, page) = Pagination::normalize(None, None);
        assert_eq!(limit, 10);
        assert_eq!(offset, 0);
        assert_eq!(page, 1);
    }

    #[test]
    fn correct_offset_for_page_two() {
        let (limit, offset, page) = Pagination::normalize(Some(2), Some(20));
        assert_eq!(limit, 20);
        assert_eq!(offset, 20); // (2-1)*20
        assert_eq!(page, 2);
    }

    #[test]
    fn page_less_than_one_clamps_to_one() {
        let (_, offset, page) = Pagination::normalize(Some(0), None);
        assert_eq!(page, 1);
        assert_eq!(offset, 0);
    }

    #[test]
    fn page_size_clamps_to_max_100() {
        let (limit, _, _) = Pagination::normalize(None, Some(999));
        assert_eq!(limit, 100);
    }

    #[test]
    fn page_size_clamps_to_min_1() {
        let (limit, _, _) = Pagination::normalize(None, Some(0));
        assert_eq!(limit, 1);
    }
}

use crate::client::{BiliClient, BiliClientInner, BiliResponse, FormBuilder, Params};
use crate::error::BiliError;
use crate::types::{HistoryResult, ToviewResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct HistoryClient {
    client: BiliClient,
}

impl HistoryClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取观看历史记录
    ///
    /// `GET https://api.bilibili.com/x/web-interface/history/cursor`
    ///
    /// 该接口使用游标而非页码翻页：把上一次响应的 `data.cursor`
    /// 中的 `max` / `view_at` / `business` 原样传回来即可获取下一页。
    ///
    /// - `max`: 历史记录截止目标 id（默认 0）
    /// - `view_at`: 历史记录截止时间戳（默认 0，表示当前时间）
    /// - `business`: 截止目标业务类型（`archive` / `pgc` / `live` / `article` 等）
    /// - `ps`: 每页项数（默认 20，最大 30）
    pub async fn list(
        &self,
        max: Option<i64>,
        view_at: Option<i64>,
        business: Option<&str>,
        ps: Option<i64>,
    ) -> Result<BiliResponse<HistoryResult>, BiliError> {
        let mut params = Params::new()
            .push_opt_val("max", max)
            .push_opt_val("view_at", view_at)
            .push_opt("business", business);
        params = params.push_val("ps", ps.unwrap_or(20));
        let mut resp = self
            .client
            .get(
                "https://api.bilibili.com/x/web-interface/history/cursor",
                params,
            )
            .await?;
        if let Some(data) = &mut resp.data {
            drop_cursor_item(data, max, view_at);
        }
        Ok(resp)
    }

    /// 删除观看历史记录
    ///
    /// `POST https://api.bilibili.com/x/web-interface/history/del`
    ///
    /// - `aid`: 视频 avid
    pub async fn delete(&self, aid: i64) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("aid", aid)
            .push("history_type", "archive")
            .push("type", "del")
            .csrf(csrf)
            .build();

        self.client
            .post(
                "https://api.bilibili.com/x/web-interface/history/del",
                &form,
            )
            .await
    }

    /// 清空观看历史记录
    ///
    /// `POST https://api.bilibili.com/x/web-interface/history/clear`
    pub async fn clear(&self) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new().push("type", "clear").csrf(csrf).build();

        self.client
            .post(
                "https://api.bilibili.com/x/web-interface/history/clear",
                &form,
            )
            .await
    }

    /// 获取稍后再看列表
    ///
    /// `GET https://api.bilibili.com/x/v2/history/toview`
    pub async fn toview_list(&self) -> Result<BiliResponse<ToviewResult>, BiliError> {
        self.client
            .get(
                "https://api.bilibili.com/x/v2/history/toview",
                Params::new(),
            )
            .await
    }

    /// 添加视频到稍后再看
    ///
    /// `POST https://api.bilibili.com/x/v2/history/toview/add`
    ///
    /// - `aid`: 视频 avid (与 bvid 任选一个)
    /// - `bvid`: 视频 bvid (与 aid 任选一个)
    pub async fn toview_add(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;

        let mut form = FormBuilder::new().csrf(csrf);
        if let Some(aid) = aid {
            form = form.push_val("aid", aid);
        }
        if let Some(bvid) = bvid {
            form = form.push("bvid", bvid);
        }

        self.client
            .post(
                "https://api.bilibili.com/x/v2/history/toview/add",
                &form.build(),
            )
            .await
    }

    /// 从稍后再看中删除视频
    ///
    /// `POST https://api.bilibili.com/x/v2/history/toview/del`
    ///
    /// - `aid`: 视频 avid (删除指定记录)
    /// - `viewed`: 是否删除已观看的 (true=删除所有已观看的)
    pub async fn toview_del(
        &self,
        aid: Option<i64>,
        viewed: Option<bool>,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;

        let mut form = FormBuilder::new().csrf(csrf);
        if let Some(aid) = aid {
            form = form.push_val("aid", aid);
        }
        if let Some(true) = viewed {
            form = form.push("viewed", "true");
        }

        self.client
            .post(
                "https://api.bilibili.com/x/v2/history/toview/del",
                &form.build(),
            )
            .await
    }

    /// 清空稍后再看列表
    ///
    /// `POST https://api.bilibili.com/x/v2/history/toview/clear`
    pub async fn toview_clear(&self) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new().csrf(csrf).build();

        self.client
            .post("https://api.bilibili.com/x/v2/history/toview/clear", &form)
            .await
    }
}

/// Remove the boundary record that Bilibili repeats when requesting the next
/// cursor page (it returns the item at `max`/`view_at` again as the first row).
fn drop_cursor_item(result: &mut HistoryResult, max: Option<i64>, view_at: Option<i64>) {
    if max.is_none() && view_at.is_none() {
        return;
    }
    let is_boundary = result.list.first().is_some_and(|item| {
        max.is_none_or(|max| item.history.oid == max)
            && view_at.is_none_or(|view_at| item.view_at == view_at)
    });
    if is_boundary {
        result.list.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HistoryItem, HistoryItemHistory};

    fn item(oid: i64, view_at: i64) -> HistoryItem {
        HistoryItem {
            view_at,
            history: HistoryItemHistory {
                oid,
                ..HistoryItemHistory::default()
            },
            ..HistoryItem::default()
        }
    }

    fn result(items: Vec<HistoryItem>) -> HistoryResult {
        HistoryResult {
            list: items,
            ..HistoryResult::default()
        }
    }

    #[test]
    fn drops_repeated_cursor_item() {
        let mut history = result(vec![item(10, 100), item(9, 99)]);
        drop_cursor_item(&mut history, Some(10), Some(100));
        assert_eq!(history.list.len(), 1);
        assert_eq!(history.list[0].history.oid, 9);
    }

    #[test]
    fn keeps_first_page_and_non_matching_items() {
        let mut first_page = result(vec![item(10, 100), item(9, 99)]);
        drop_cursor_item(&mut first_page, None, None);
        assert_eq!(first_page.list.len(), 2);

        let mut different = result(vec![item(11, 101), item(10, 100)]);
        drop_cursor_item(&mut different, Some(10), Some(100));
        assert_eq!(different.list.len(), 2);
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub base_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteCreateArgs {
    pub name: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteUpdateArgs {
    pub id: i64,
    pub name: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdArgs {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub id: i64,
    pub site_id: i64,
    pub site_profile_id: Option<i64>,
    pub title: String,
    pub author_name: Option<String>,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteProfile {
    pub id: i64,
    pub site_id: i64,
    pub name: String,
    pub profile_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkCreateArgs {
    pub site_id: i64,
    pub site_profile_id: Option<i64>,
    pub title: String,
    pub author_name: Option<String>,
    pub description: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkUpdateArgs {
    pub id: i64,
    pub site_profile_id: Option<i64>,
    pub title: String,
    pub author_name: Option<String>,
    pub description: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkListBySiteArgs {
    pub site_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteProfileCreateArgs {
    pub site_id: i64,
    pub name: String,
    pub profile_json: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteProfileUpdateArgs {
    pub id: i64,
    pub name: String,
    pub profile_json: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteProfileListArgs {
    pub site_id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: i64,
    pub work_id: i64,
    pub page_number: Option<i64>,
    pub sort_order: i64,
    pub title: Option<String>,
    pub source_url: Option<String>,
    pub source_type: String,
    pub canonical_url: Option<String>,
    pub archived_at: Option<String>,
    pub requested_encoding: Option<String>,
    pub detected_encoding: Option<String>,
    pub content_text: Option<String>,
    pub content_html_path: Option<String>,
    pub fetch_status: String,
    pub fetch_error: Option<String>,
    pub fetched_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageCreateArgs {
    pub work_id: i64,
    pub page_number: Option<i64>,
    pub title: Option<String>,
    pub source_url: Option<String>,
    pub source_type: String,
    pub requested_encoding: Option<String>,
    pub content_text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageUpdateArgs {
    pub id: i64,
    pub page_number: Option<i64>,
    pub title: Option<String>,
    pub source_url: Option<String>,
    pub source_type: String,
    pub requested_encoding: Option<String>,
    pub content_text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageListByWorkArgs {
    pub work_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleSearchResults {
    pub sites: Vec<Site>,
    pub works: Vec<WorkSearchItem>,
    pub pages: Vec<PageSearchItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkSearchItem {
    pub id: i64,
    pub site_id: i64,
    pub site_name: String,
    pub title: String,
    pub author_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageSearchItem {
    pub id: i64,
    pub work_id: i64,
    pub work_title: String,
    pub site_id: i64,
    pub site_name: String,
    pub title: Option<String>,
    pub page_number: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchTitlesArgs {
    pub query: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullTextSearchItem {
    pub page_id: i64,
    pub page_title: Option<String>,
    pub page_number: Option<i64>,
    pub work_id: i64,
    pub work_title: String,
    pub site_id: i64,
    pub site_name: String,
    pub snippet: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFullTextArgs {
    pub query: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchPageByUrlArgs {
    pub page_id: i64,
    pub url: String,
    pub source_type: String,
    pub title_selector: String,
    pub content_selector: String,
    pub remove_selectors: Vec<String>,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorite {
    pub id: i64,
    pub page_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteListItem {
    pub favorite_id: i64,
    pub favorited_at: String,
    pub page_id: i64,
    pub page_title: Option<String>,
    pub page_number: Option<i64>,
    pub work_id: i64,
    pub work_title: String,
    pub site_id: i64,
    pub site_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoritesGrouped {
    pub groups: Vec<FavoriteSiteGroup>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteSiteGroup {
    pub site_id: i64,
    pub site_name: String,
    pub works: Vec<FavoriteWorkGroup>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteWorkGroup {
    pub work_id: i64,
    pub work_title: String,
    pub pages: Vec<FavoriteListItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoritePageArgs {
    pub page_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteCheckResult {
    pub is_favorite: bool,
    pub favorite_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkFetchByProfileArgs {
    pub work_id: i64,
    pub site_profile_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkFetchResult {
    pub created_count: usize,
    pub success_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileOutputResult {
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSourceUrlPage {
    pub page_id: i64,
    pub page_title: Option<String>,
    pub page_number: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSourceUrlGroup {
    pub site_id: i64,
    pub site_name: String,
    pub work_id: i64,
    pub work_title: String,
    pub source_type: String,
    pub source_url: String,
    pub pages: Vec<DuplicateSourceUrlPage>,
}

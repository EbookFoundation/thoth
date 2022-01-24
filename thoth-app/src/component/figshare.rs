use md5::{Digest, Md5};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thoth_api::model::subject::SubjectType;
use thoth_api::model::work::WorkType;
use thoth_api::model::work::WorkWithRelations;
use yew::html;
use yew::prelude::*;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchError;
use yewtil::fetch::FetchRequest;
use yewtil::fetch::FetchState;
use yewtil::fetch::Json;
use yewtil::fetch::MethodBody;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

// Test instance. Production instance is "https://api.figshare.com/v2".
const FIGSHARE_API_ROOT: &str = "https://api.figsh.com/v2";

// Upload API is separate from main API. Unclear whether this value
// may change - if so, should be obtained from main API responses.
const FIGSHARE_UPLOAD_API_ROOT: &str = "https://fup1010100.figsh.com/upload/";

// Authorization token associated with a Figshare user account.
// The token itself is security information and must not be published in open-source code.
// Instead, set it as an environment variable in the shell before starting the Thoth app
// (`export FIGSHARE_TOKEN=[value]`).
const FIGSHARE_TOKEN: Option<&str> = option_env!("FIGSHARE_TOKEN");

// Temporary hard-coding of single Figshare article ID for basic test purposes.
// If required, set it as an environment variable, as above for FIGSHARE_TOKEN.
const TEST_ARTICLE_ID: Option<&str> = option_env!("FIGSHARE_ARTICLE_ID");

// Child object of ArticleCreate representing an author.
// Note that this will be transformed in the created article into an Author object
// (with attributes id, full_name, is_active, url_name and orcid_id).
// url_name will default to "_" if no valid Figshare author ID is supplied.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]

pub struct FigArticleCreateAuthor {
    // This information (Figshare author ID) is not stored in Thoth.
    // pub id: String,
    // pub first_name: String,
    // pub last_name: String,
    pub name: String,
    // pub email: String,
    // This information is stored in Thoth but not currently accessible via the Work page.
    // pub orcid_id: String,
}

// This will be transformed on creation into a FundingInformation object
// (with attributes id, title, grant_code, funder_name, is_user_defined, url).
// Thoth stores information such as grant number and funder (institution) name
// but these cannot be submitted here.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigFundingCreate {
    // This appears to be a Figshare funding ID and is not stored in Thoth.
    // pub id: String,
    // Defined as "the funding name"; Thoth stores program, project name, etc.
    pub title: String,
}

// Note: once a timeline has been created, it does not seem to be possible
// to remove it (submitting empty attribute strings and empty
// TimelineUpdate objects both had no effect).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FigTimelineUpdate {
    // pub first_online: String,
    // Omit this attribute if no publication date exists (i.e. create empty object).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_publication: Option<String>,
    // pub publisher_acceptance: String,
}

// Can also be used to represent ArticleUpdate, as the objects are identical.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigArticleCreate {
    // Required fields for article creation:
    pub title: String,
    // Required fields for article publication:
    pub description: String,
    pub authors: Vec<FigArticleCreateAuthor>,
    // Figshare IDs representing ANZSRC FoR categories - TBD how to map to Thoth categories
    // pub categories: Vec<i32>,
    pub defined_type: String,
    // Transformed into "tags" on creation - consider renaming
    pub keywords: Vec<String>,
    // Figshare ID - TODO retrieve options from private licences endpoint,
    // match option URL to licence URL stored in Thoth, submit corresponding ID.
    // pub license: i32,
    // (A subset of) optional fields:
    pub funding_list: Vec<FigFundingCreate>,
    pub timeline: FigTimelineUpdate,
    pub resource_doi: String,
}

#[derive(Debug, Clone, Default)]
pub struct FigArticleUpdateRequest {
    pub body: FigArticleCreate,
}

// Standard Figshare response to API request (article create/update)
// appears to consist of "location" (of article) and "warnings";
// however, error responses seem to contain "message" and "code" instead.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigshareResponseBody {
    pub location: String,
    pub warnings: Vec<String>,
}

// Implement Yewtil's example template for reducing HTTP request boilerplate
// (see documentation for FetchRequest)
pub trait SlimFetchRequest {
    type RequestBody: Serialize;
    type ResponseBody: DeserializeOwned;
    fn path(&self) -> String;
    fn method(&self) -> MethodBody<Self::RequestBody>;
    // Default to main API - can be overridden
    fn root(&self) -> String {
        FIGSHARE_API_ROOT.to_string()
    }
    // Default to creating URL from root + path - can be overridden
    fn full_url(&self) -> String {
        format!("{}{}", self.root(), self.path())
    }
}

#[derive(Default)]
pub struct FetchWrapper<T>(T);

impl<T: SlimFetchRequest> FetchRequest for FetchWrapper<T> {
    type RequestBody = T::RequestBody;
    type ResponseBody = T::ResponseBody;
    type Format = Json;

    fn url(&self) -> String {
        self.0.full_url()
    }

    fn method(&self) -> MethodBody<Self::RequestBody> {
        self.0.method()
    }

    // Write requests require authentication information and a JSON body containing the data to be written.
    fn headers(&self) -> Vec<(String, String)> {
        let json = ("Content-Type".into(), "application/json".into());
        let auth = (
            "Authorization".into(),
            format!("token {}", FIGSHARE_TOKEN.unwrap()),
        );
        vec![json, auth]
    }

    fn use_cors(&self) -> bool {
        false
    }
}

impl SlimFetchRequest for FigArticleUpdateRequest {
    type RequestBody = FigArticleCreate;
    type ResponseBody = FigshareResponseBody;
    fn path(&self) -> String {
        // Endpoint for updating existing article.
        format!("/account/articles/{}", TEST_ARTICLE_ID.unwrap())
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        // Updates use HTTP method PUT.
        MethodBody::Put(&self.body)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigFileCreator {
    pub md5: String,
    pub name: String,
    pub size: i32,
    // Should never be filled out - stores external link without saving its content
    // pub link: String,
}

#[derive(Debug, Clone, Default)]
pub struct FigUploadGetIdRequest {
    pub body: FigFileCreator,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigUploadGetIdResponse {
    pub location: String,
}

impl SlimFetchRequest for FigUploadGetIdRequest {
    type RequestBody = FigFileCreator;
    type ResponseBody = FigUploadGetIdResponse;
    fn path(&self) -> String {
        format!("/account/articles/{}/files", TEST_ARTICLE_ID.unwrap())
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&self.body)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FigUploadGetUrlRequest {
    // Previous response contains full URL. Plain file ID not easily extracted.
    // pub file_id: String,
    pub location: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigUploadGetUrlResponse {
    pub upload_token: String,
    pub upload_url: String,
    pub status: String,
    pub preview_state: String,
    pub viewer_type: String,
    pub id: i32,
    pub name: String,
    pub size: i32,
    pub is_link_only: bool,
    pub download_url: String,
    pub supplied_md5: String,
    pub computed_md5: String,
}

impl SlimFetchRequest for FigUploadGetUrlRequest {
    type RequestBody = ();
    type ResponseBody = FigUploadGetUrlResponse;
    // Override default root + path URL with full URL from previous response.
    // `path()` will not be used but must be implemented.
    // Alternatively, extract plain file ID and omit `full_url()`,
    // using commented-out version of `path()` below.
    fn full_url(&self) -> String {
        self.location.clone()
    }
    fn path(&self) -> String {
        "unimplemented".to_string()
    }
    // fn path(&self) -> String {
    //     format!("/account/articles/{}/files/{}",
    //     TEST_ARTICLE_ID.unwrap(),
    //     &self.file_id)
    // }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Get
    }
}

#[derive(Debug, Clone, Default)]
pub struct FigUploadGetPartsRequest {
    pub upload_token: String,
    // pub upload_url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigUploadGetPartsResponse {
    pub token: String,
    pub name: String,
    pub size: i32,
    pub md5: String,
    pub status: String,
    pub parts: Vec<FigUploadPartData>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FigUploadPartData {
    pub part_no: i32,
    pub start_offset: i32,
    pub end_offset: i32,
    pub status: String,
    pub locked: bool,
}

impl SlimFetchRequest for FigUploadGetPartsRequest {
    type RequestBody = ();
    type ResponseBody = FigUploadGetPartsResponse;
    fn root(&self) -> String {
        FIGSHARE_UPLOAD_API_ROOT.to_string()
    }
    fn path(&self) -> String {
        self.upload_token.to_string()
    }
    // Previous response contains both upload_url (root + upload_token)
    // and plain upload_token. Alternative implementation uses full URL:
    // fn full_url(&self) -> String { &self.upload_url }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Get
    }
}

#[derive(Debug, Clone, Default)]
pub struct FigUploadSendPartRequest {
    pub upload_token: String,
    pub part_no: String,
    pub body: Vec<u8>,
}

impl SlimFetchRequest for FigUploadSendPartRequest {
    type RequestBody = Vec<u8>;
    // Body is not actually empty but contains plain text "OK" (if success -
    // may be a JSON-formatted error message otherwise).
    // Fetch framework expects JSON body so we cannot easily set appropriate type.
    type ResponseBody = ();
    fn root(&self) -> String {
        FIGSHARE_UPLOAD_API_ROOT.to_string()
    }
    fn path(&self) -> String {
        format!("{}/{}", self.upload_token, self.part_no)
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Put(&self.body)
    }
}

// Note: structure identical to FigUploadGetUrlRequest
// (but this cannot be reused as the SlimFetchRequest impl needs to be different).
#[derive(Debug, Clone, Default)]
pub struct FigUploadResultRequest {
    // Previous response contains full URL. Plain file ID not easily extracted.
    // pub file_id: String,
    pub location: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigUploadResultRequestBody {}

impl SlimFetchRequest for FigUploadResultRequest {
    // API requires a POST with empty body.
    // Unclear how to do this within Fetch framework.
    // Send dummy struct - this is successful as API ignores body.
    type RequestBody = FigUploadResultRequestBody;
    // Body is not actually empty but contains HTML "Accepted" message (if success -
    // may be a JSON-formatted error message otherwise).
    // Fetch framework expects JSON body so we cannot easily set appropriate type.
    type ResponseBody = ();
    // Override default root + path URL with full URL from previous response.
    // `path()` will not be used but must be implemented.
    // Alternatively, extract plain file ID and omit `full_url()`,
    // using commented-out version of `path()` below.
    // (See also FigUploadGetUrlRequest.)
    fn full_url(&self) -> String {
        self.location.clone()
    }
    fn path(&self) -> String {
        "unimplemented".to_string()
    }
    // fn path(&self) -> String {
    //     format!("/account/articles/{}/files/{}",
    //     TEST_ARTICLE_ID.unwrap(),
    //     &self.file_id)
    // }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&FigUploadResultRequestBody {})
    }
}

pub type PushFigshareRequest = Fetch<FetchWrapper<FigArticleUpdateRequest>, FigshareResponseBody>;
pub type PushActionFigshareRequest = FetchAction<FigshareResponseBody>;
pub type UploadGetIdRequest = Fetch<FetchWrapper<FigUploadGetIdRequest>, FigUploadGetIdResponse>;
pub type UploadActionGetIdRequest = FetchAction<FigUploadGetIdResponse>;
pub type UploadGetUrlRequest = Fetch<FetchWrapper<FigUploadGetUrlRequest>, FigUploadGetUrlResponse>;
pub type UploadActionGetUrlRequest = FetchAction<FigUploadGetUrlResponse>;
pub type UploadGetPartsRequest =
    Fetch<FetchWrapper<FigUploadGetPartsRequest>, FigUploadGetPartsResponse>;
pub type UploadActionGetPartsRequest = FetchAction<FigUploadGetPartsResponse>;
pub type UploadSendPartRequest = Fetch<FetchWrapper<FigUploadSendPartRequest>, ()>;
pub type UploadActionSendPartRequest = FetchAction<()>;
pub type UploadResultRequest = Fetch<FetchWrapper<FigUploadResultRequest>, ()>;
pub type UploadActionResultRequest = FetchAction<()>;

// Basic interface: triggers conversion of Thoth Work data into Figshare Article format
// and sends write request with formatted data to Figshare endpoint.

pub struct FigshareComponent {
    props: Props,
    link: ComponentLink<Self>,
    push_figshare: PushFigshareRequest,
    upload_get_id: UploadGetIdRequest,
    upload_get_url: UploadGetUrlRequest,
    upload_get_parts: UploadGetPartsRequest,
    upload_send_part: UploadSendPartRequest,
    upload_get_result: UploadResultRequest,
    file_location: String,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub work: WorkWithRelations,
}

pub enum Msg {
    SetFigsharePushState(PushActionFigshareRequest),
    Submit,
    InitiateFigshareUpload,
    GetFigshareFileId(UploadActionGetIdRequest),
    GetFigshareUploadUrl(UploadActionGetUrlRequest),
    GetFigshareUploadParts(UploadActionGetPartsRequest),
    ConcludeFigshareUpload(UploadActionSendPartRequest),
    GetFigshareUploadResult(UploadActionResultRequest),
}

impl Component for FigshareComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_figshare = Default::default();
        let upload_get_id = Default::default();
        let upload_get_url = Default::default();
        let upload_get_parts = Default::default();
        let upload_send_part = Default::default();
        let upload_get_result = Default::default();
        let file_location = Default::default();
        FigshareComponent {
            props,
            link,
            push_figshare,
            upload_get_id,
            upload_get_url,
            upload_get_parts,
            upload_send_part,
            upload_get_result,
            file_location,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props);
        // Appearance of component is currently static, so no need to re-render
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetFigsharePushState(fetch_state) => {
                self.push_figshare.apply(fetch_state);
                // TODO: process response received from Figshare
                false
            }
            Msg::Submit => {
                let mut authors = vec![];
                for contribution in self.props.work.contributions.clone().unwrap_or_default() {
                    let author = FigArticleCreateAuthor {
                        name: contribution.full_name,
                        // Stored in Thoth, but not currently requested when retrieving Work
                        // orcid_id: contribution.contributor.orcid.unwrap_or_default(),
                    };
                    authors.push(author);
                }
                // Options as listed in documentation are:
                // figure | online resource | preprint | book | conference contribution
                // media | dataset | poster | journal contribution | presentation | thesis | software
                // However, options from ArticleSearch item_type full list also seem to be accepted:
                // 1 - Figure, 2 - Media, 3 - Dataset, 5 - Poster, 6 - Journal contribution, 7 - Presentation,
                // 8 - Thesis, 9 - Software, 11 - Online resource, 12 - Preprint, 13 - Book, 14 - Conference contribution,
                // 15 - Chapter, 16 - Peer review, 17 - Educational resource, 18 - Report, 19 - Standard, 20 - Composition,
                // 21 - Funding, 22 - Physical object, 23 - Data management plan, 24 - Workflow, 25 - Monograph,
                // 26 - Performance, 27 - Event, 28 - Service, 29 - Model
                let defined_type = match self.props.work.work_type {
                    WorkType::BookChapter => "chapter".to_string(),
                    WorkType::Monograph => "monograph".to_string(),
                    WorkType::EditedBook => "book".to_string(),
                    WorkType::Textbook => "educational resource".to_string(),
                    WorkType::JournalIssue => "book".to_string(),
                    WorkType::BookSet => "book".to_string(),
                };
                let keywords = self
                    .props
                    .work
                    .subjects
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .filter(|s| s.subject_type.eq(&SubjectType::Keyword))
                    .map(|s| s.subject_code.clone())
                    .collect();
                let fundings: Vec<String> = self
                    .props
                    .work
                    .fundings
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    // Unclear which attribute to use as "the funding name"; use grant number for now.
                    // (Will omit fundings with no grant number.)
                    .filter_map(|f| f.grant_number.clone())
                    .collect();
                let mut funding_list = vec![];
                for funding in fundings {
                    funding_list.push(FigFundingCreate { title: funding });
                }
                let body = FigArticleCreate {
                    title: self.props.work.full_title.clone(),
                    description: self.props.work.long_abstract.clone().unwrap_or_default(),
                    authors,
                    defined_type,
                    keywords,
                    funding_list,
                    timeline: FigTimelineUpdate {
                        publisher_publication: self.props.work.publication_date.clone(),
                    },
                    // Supplied without leading "https://doi.org/".
                    // If empty, will submit "" and clear any previous value.
                    resource_doi: self.props.work.doi.clone().unwrap_or_default().to_string(),
                };
                let request = FetchWrapper(FigArticleUpdateRequest { body });
                self.push_figshare = Fetch::new(request);
                self.link
                    .send_future(self.push_figshare.fetch(Msg::SetFigsharePushState));
                self.link
                    .send_message(Msg::SetFigsharePushState(FetchAction::Fetching));
                false
            }
            Msg::InitiateFigshareUpload => {
                // POST to /articles/{article_id}/files
                // JSON body: "md5", "name", "size"
                // Calculate MD5 hash of file to be uploaded
                let mut hasher = Md5::new();
                // Hard-coded temporary test data
                hasher.update(b"12345");
                let hash = hasher.finalize();
                let md5 = format!("{:x}", hash);
                let body = FigFileCreator {
                    md5,
                    name: "name".to_string(),
                    size: 5,
                };
                let request = FetchWrapper(FigUploadGetIdRequest { body });
                self.upload_get_id = Fetch::new(request);
                self.link
                    .send_future(self.upload_get_id.fetch(Msg::GetFigshareFileId));
                self.link
                    .send_message(Msg::GetFigshareFileId(FetchAction::Fetching));
                false
            }
            Msg::GetFigshareFileId(fetch_state) => {
                self.upload_get_id.apply(fetch_state);
                match self.upload_get_id.as_ref().state() {
                    FetchState::Fetched(body) => {
                        // Response contains full URL (in format root/articles/{article_id}/files/{file_id}).
                        // Save off for use when confirming upload completed.
                        // Alternatively we could extract and save the plain file ID.
                        self.file_location = body.location.clone();
                        // GET from /articles/{article_id}/files/{file_id}
                        // JSON body: none
                        let request = FetchWrapper(FigUploadGetUrlRequest {
                            // file_id: self.file_id.clone()
                            location: self.file_location.clone(),
                        });
                        self.upload_get_url = Fetch::new(request);
                        self.link
                            .send_future(self.upload_get_url.fetch(Msg::GetFigshareUploadUrl));
                        self.link
                            .send_message(Msg::GetFigshareUploadUrl(FetchAction::Fetching));
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::GetFigshareUploadUrl(fetch_state) => {
                self.upload_get_url.apply(fetch_state);
                match self.upload_get_url.as_ref().state() {
                    FetchState::Fetched(body) => {
                        // Response contains full upload_url (in format upload_root/{upload_token})
                        // and, separately, plain upload_token. Could alternatively extract full URL.
                        // GET from [upload API root]/{upload_token} (separate from main Figshare API)
                        // JSON body: none
                        let request = FetchWrapper(FigUploadGetPartsRequest {
                            // upload_url: body.upload_url.clone()
                            upload_token: body.upload_token.clone(),
                        });
                        self.upload_get_parts = Fetch::new(request);
                        self.link
                            .send_future(self.upload_get_parts.fetch(Msg::GetFigshareUploadParts));
                        self.link
                            .send_message(Msg::GetFigshareUploadParts(FetchAction::Fetching));
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::GetFigshareUploadParts(fetch_state) => {
                self.upload_get_parts.apply(fetch_state);
                match self.upload_get_parts.as_ref().state() {
                    FetchState::Fetched(body) => {
                        // Response contains upload token (again), and set of parts into
                        // which data needs to be split (inc. part_no and start/end offsets).
                        // For each part:
                        // PUT to [upload API root]/{upload_token}/{part_no}
                        // JSON body: raw file data
                        // TODO: add support for multi-part files, including calculating offsets
                        // (currently only tested and working for files of exactly one part)
                        for part in &body.parts {
                            let request = FetchWrapper(FigUploadSendPartRequest {
                                upload_token: body.token.clone(),
                                part_no: part.part_no.to_string(),
                                // Hard-coded temporary test data
                                body: "12345".as_bytes().to_owned(),
                            });
                            self.upload_send_part = Fetch::new(request);
                            self.link.send_future(
                                self.upload_send_part.fetch(Msg::ConcludeFigshareUpload),
                            );
                            self.link
                                .send_message(Msg::ConcludeFigshareUpload(FetchAction::Fetching));
                        }
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::ConcludeFigshareUpload(fetch_state) => {
                self.upload_send_part.apply(fetch_state);
                match self.upload_send_part.as_ref().state() {
                    // Workaround for handling Figshare 200 OK response with
                    // plain text body "OK": Fetch logic expects JSON body
                    // (not trivial to change) therefore fails to handle.
                    // If the body text is "OK" as expected, assume success.
                    FetchState::Failed(_body, fetch_error) => {
                        if let FetchError::DeserializeError { error: _, content } = fetch_error {
                            if content.eq(&"OK".to_string()) {
                                // To mark the upload as completed:
                                // POST to /articles/{article_id}/files/{file_id}
                                // JSON body: none
                                // TODO: in practice, need to wait until all parts have successfully been uploaded.
                                let request = FetchWrapper(FigUploadResultRequest {
                                    // file_id: self.file_id.clone()
                                    location: self.file_location.clone(),
                                });
                                self.upload_get_result = Fetch::new(request);
                                self.link.send_future(
                                    self.upload_get_result.fetch(Msg::GetFigshareUploadResult),
                                );
                                self.link.send_message(Msg::GetFigshareUploadResult(
                                    FetchAction::Fetching,
                                ));
                            }
                            // TODO handle other errors
                        }
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Fetched(_) => (),
                }
                false
            }
            Msg::GetFigshareUploadResult(fetch_state) => {
                self.upload_get_result.apply(fetch_state);
                // TODO: process response received from Figshare
                false
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <>
                <button onclick=self.link.callback(|_| Msg::Submit)>
                    { "Submit to Figshare" }
                </button>
                <button onclick=self.link.callback(|_| Msg::InitiateFigshareUpload)>
                    { "Upload test file" }
                </button>
            </>
        }
    }
}
use crate::models::contributor::contributors_query::ContributorsRequest;
use crate::models::contributor::contributors_query::ContributorsRequestBody;
use crate::models::contributor::contributors_query::FetchActionContributors;
use crate::models::contributor::contributors_query::FetchContributors;
use crate::models::contributor::contributors_query::Variables;
use crate::models::contributor::Contributor;
use thoth_api::contributor::model::ContributorField;
use thoth_api::contributor::model::ContributorOrderBy;

pagination_component! {
    ContributorsComponent,
    Contributor,
    contributors,
    contributor_count,
    ContributorsRequest,
    FetchActionContributors,
    FetchContributors,
    ContributorsRequestBody,
    Variables,
    SEARCH_CONTRIBUTORS,
    PAGINATION_COUNT_CONTRIBUTORS,
    vec![
        ContributorField::ContributorID.to_string(),
        ContributorField::FullName.to_string(),
        ContributorField::ORCID.to_string(),
        ContributorField::UpdatedAt.to_string(),
    ],
    ContributorOrderBy,
    ContributorField,
}

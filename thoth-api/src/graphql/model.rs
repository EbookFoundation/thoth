use chrono::naive::NaiveDate;
use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::RootNode;
use std::sync::Arc;
use uuid::Uuid;

use crate::account::model::AccountAccess;
use crate::account::model::DecodedToken;
use crate::contribution::model::*;
use crate::contributor::model::*;
use crate::db::PgPool;
use crate::errors::ThothError;
use crate::errors::ThothResult;
use crate::funder::model::*;
use crate::funding::model::*;
use crate::imprint::model::*;
use crate::issue::model::*;
use crate::language::model::*;
use crate::model::Crud;
use crate::price::model::*;
use crate::publication::model::*;
use crate::publisher::model::*;
use crate::schema::*;
use crate::series::model::*;
use crate::subject::model::*;
use crate::work::model::*;

use super::utils::Direction;

impl juniper::Context for Context {}

#[derive(Clone)]
pub struct Context {
    pub db: Arc<PgPool>,
    pub account_access: AccountAccess,
    pub token: DecodedToken,
}

impl Context {
    pub fn new(pool: Arc<PgPool>, token: DecodedToken) -> Self {
        Self {
            db: pool,
            account_access: token.get_user_permissions(),
            token,
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting contributions list")]
pub struct ContributionOrderBy {
    pub field: ContributionField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting issues list")]
pub struct IssueOrderBy {
    pub field: IssueField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting languages list")]
pub struct LanguageOrderBy {
    pub field: LanguageField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting prices list")]
pub struct PriceOrderBy {
    pub field: PriceField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting subjects list")]
pub struct SubjectOrderBy {
    pub field: SubjectField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting fundings list")]
pub struct FundingOrderBy {
    pub field: FundingField,
    pub direction: Direction,
}

pub struct QueryRoot;

#[juniper::object(Context = Context)]
impl QueryRoot {
    #[graphql(
    description="Query the full list of works",
    arguments(
        limit(
            default = 100,
            description = "The number of items to return"
        ),
        offset(
            default = 0,
            description = "The number of items to skip"
        ),
        filter(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        ),
        order(
            default = WorkOrderBy::default(),
            description = "The order in which to sort the results",
        ),
        publishers(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs",
        ),
        work_type(description = "A specific type to filter by"),
        work_status(description = "A specific status to filter by"),
    )
  )]
    fn works(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: WorkOrderBy,
        publishers: Vec<Uuid>,
        work_type: Option<WorkType>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<Vec<Work>> {
        Work::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            work_type,
            work_status,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single work using its id")]
    fn work(context: &Context, work_id: Uuid) -> FieldResult<Work> {
        Work::from_id(&context.db, &work_id).map_err(|e| e.into())
    }

    #[graphql(description = "Query a single work using its DOI")]
    fn work_by_doi(context: &Context, doi: String) -> FieldResult<Work> {
        let connection = context.db.get().unwrap();
        use diesel::sql_types::Nullable;
        use diesel::sql_types::Text;
        // Allow case-insensitive searching (DOIs in database may have mixed casing)
        sql_function!(fn lower(x: Nullable<Text>) -> Nullable<Text>);
        crate::schema::work::dsl::work
            .filter(lower(crate::schema::work::dsl::doi).eq(doi.to_lowercase()))
            .get_result::<Work>(&connection)
            .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of works",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            work_type(description = "A specific type to filter by"),
            work_status(description = "A specific status to filter by"),
        )
    )]
    fn work_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        work_type: Option<WorkType>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<i32> {
        Work::count(
            &context.db,
            Some(filter),
            publishers,
            work_type,
            work_status,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of publications",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn and publication_url"
            ),
            order(
                default = PublicationOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            publication_type(description = "A specific type to filter by"),
        )
    )]
    fn publications(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: PublicationOrderBy,
        publishers: Vec<Uuid>,
        publication_type: Option<PublicationType>,
    ) -> FieldResult<Vec<Publication>> {
        let connection = context.db.get().unwrap();
        Publication::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            publication_type,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single publication using its id")]
    fn publication(context: &Context, publication_id: Uuid) -> FieldResult<Publication> {
        Publication::from_id(&context.db, &publication_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of publications",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn and publication_url",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            publication_type(description = "A specific type to filter by"),
        )
    )]
    fn publication_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        publication_type: Option<PublicationType>,
    ) -> FieldResult<i32> {
        Publication::count(
            &context.db,
            Some(filter),
            publishers,
            publication_type,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
    description="Query the full list of publishers",
    arguments(
        limit(
            default = 100,
            description = "The number of items to return"
        ),
        offset(
            default = 0,
            description = "The number of items to skip"
        ),
        filter(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on publisher_name and publisher_shortname"

        ),
        order(
            default = PublisherOrderBy::default(),
            description = "The order in which to sort the results",
        ),
        publishers(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs",
        ),
    )
  )]
    fn publishers(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: PublisherOrderBy,
        publishers: Vec<Uuid>,
    ) -> FieldResult<Vec<Publisher>> {
        let connection = context.db.get().unwrap();
        Publisher::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            None,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single publisher using its id")]
    fn publisher(context: &Context, publisher_id: Uuid) -> FieldResult<Publisher> {
        Publisher::from_id(&context.db, &publisher_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of publishers",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on publisher_name and publisher_shortname",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn publisher_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
    ) -> FieldResult<i32> {
        Publisher::count(&context.db, Some(filter), publishers, None, None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of imprints",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on imprint_name and imprint_url"
            ),
            order(
                default = ImprintOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn imprints(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: ImprintOrderBy,
        publishers: Vec<Uuid>,
    ) -> FieldResult<Vec<Imprint>> {
        let connection = context.db.get().unwrap();
        Imprint::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            None,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single imprint using its id")]
    fn imprint(context: &Context, imprint_id: Uuid) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &imprint_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of imprints",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on imprint_name and imprint_url",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn imprint_count(context: &Context, filter: String, publishers: Vec<Uuid>) -> FieldResult<i32> {
        Imprint::count(&context.db, Some(filter), publishers, None, None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of contributors",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_name and orcid"
            ),
            order(
                default = ContributorOrderBy::default(),
                description = "The order in which to sort the results",
            ),
        )
    )]
    fn contributors(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: ContributorOrderBy,
    ) -> FieldResult<Vec<Contributor>> {
        Contributor::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            None,
            None,
            None,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single contributor using its id")]
    fn contributor(context: &Context, contributor_id: Uuid) -> FieldResult<Contributor> {
        Contributor::from_id(&context.db, &contributor_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of contributors",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_name and orcid",
            ),
        )
    )]
    fn contributor_count(context: &Context, filter: String) -> FieldResult<i32> {
        Contributor::count(&context.db, Some(filter), vec![], None, None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of contributions",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    ContributionOrderBy {
                        field: ContributionField::ContributionType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            contribution_type(description = "A specific type to filter by"),
        )
    )]
    fn contributions(
        context: &Context,
        limit: i32,
        offset: i32,
        order: ContributionOrderBy,
        publishers: Vec<Uuid>,
        contribution_type: Option<ContributionType>,
    ) -> Vec<Contribution> {
        use crate::schema::contribution::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::contribution
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                dsl::work_id,
                dsl::contributor_id,
                dsl::contribution_type,
                dsl::main_contribution,
                dsl::biography,
                dsl::institution,
                dsl::created_at,
                dsl::updated_at,
                dsl::first_name,
                dsl::last_name,
                dsl::full_name,
            ))
            .into_boxed();
        match order.field {
            ContributionField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            ContributionField::ContributorId => match order.direction {
                Direction::Asc => query = query.order(dsl::contributor_id.asc()),
                Direction::Desc => query = query.order(dsl::contributor_id.desc()),
            },
            ContributionField::ContributionType => match order.direction {
                Direction::Asc => query = query.order(dsl::contribution_type.asc()),
                Direction::Desc => query = query.order(dsl::contribution_type.desc()),
            },
            ContributionField::MainContribution => match order.direction {
                Direction::Asc => query = query.order(dsl::main_contribution.asc()),
                Direction::Desc => query = query.order(dsl::main_contribution.desc()),
            },
            ContributionField::Biography => match order.direction {
                Direction::Asc => query = query.order(dsl::biography.asc()),
                Direction::Desc => query = query.order(dsl::biography.desc()),
            },
            ContributionField::Institution => match order.direction {
                Direction::Asc => query = query.order(dsl::institution.asc()),
                Direction::Desc => query = query.order(dsl::institution.desc()),
            },
            ContributionField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            ContributionField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
            ContributionField::FirstName => match order.direction {
                Direction::Asc => query = query.order(dsl::first_name.asc()),
                Direction::Desc => query = query.order(dsl::first_name.desc()),
            },
            ContributionField::LastName => match order.direction {
                Direction::Asc => query = query.order(dsl::last_name.asc()),
                Direction::Desc => query = query.order(dsl::last_name.desc()),
            },
            ContributionField::FullName => match order.direction {
                Direction::Asc => query = query.order(dsl::full_name.asc()),
                Direction::Desc => query = query.order(dsl::full_name.desc()),
            },
        }
        // This loop must appear before any other filter statements, as it takes advantage of
        // the behaviour of `or_filter` being equal to `filter` when no other filters are present yet.
        // Result needs to be `WHERE (x = $1 [OR x = $2...]) AND ([...])` - note bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(cont_type) = contribution_type {
            query = query.filter(dsl::contribution_type.eq(cont_type))
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }

    #[graphql(description = "Query a single contribution using its identifiers")]
    fn contribution(
        context: &Context,
        work_id: Uuid,
        contributor_id: Uuid,
        contribution_type: ContributionType,
    ) -> FieldResult<Contribution> {
        let connection = context.db.get().unwrap();
        crate::schema::contribution::dsl::contribution
            .filter(crate::schema::contribution::dsl::work_id.eq(work_id))
            .filter(crate::schema::contribution::dsl::contributor_id.eq(contributor_id))
            .filter(crate::schema::contribution::dsl::contribution_type.eq(contribution_type))
            .get_result::<Contribution>(&connection)
            .map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of contributions")]
    fn contribution_count(context: &Context, contribution_type: Option<ContributionType>) -> i32 {
        use crate::schema::contribution::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::contribution.into_boxed();
        if let Some(cont_type) = contribution_type {
            query = query.filter(dsl::contribution_type.eq(cont_type))
        }
        // see comment in work_count()
        query
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading contribution count")
            .to_string()
            .parse::<i32>()
            .unwrap()
    }

    #[graphql(
        description = "Query the full list of series",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on series_name, issn_print, issn_digital and series_url"
            ),
            order(
                default = SeriesOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            series_type(description = "A specific type to filter by"),
        ),
    )]
    fn serieses(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: SeriesOrderBy,
        publishers: Vec<Uuid>,
        series_type: Option<SeriesType>,
    ) -> FieldResult<Vec<Series>> {
        let connection = context.db.get().unwrap();
        Series::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            series_type,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single series using its id")]
    fn series(context: &Context, series_id: Uuid) -> FieldResult<Series> {
        Series::from_id(&context.db, &series_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of series",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on series_name, issn_print, issn_digital and series_url",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            series_type(description = "A specific type to filter by"),
        )
    )]
    fn series_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        series_type: Option<SeriesType>,
    ) -> FieldResult<i32> {
        Series::count(&context.db, Some(filter), publishers, series_type, None)
            .map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of issues",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    IssueOrderBy {
                        field: IssueField::IssueOrdinal,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn issues(
        context: &Context,
        limit: i32,
        offset: i32,
        order: IssueOrderBy,
        publishers: Vec<Uuid>,
    ) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = issue
            .inner_join(crate::schema::series::table.inner_join(crate::schema::imprint::table))
            .select((series_id, work_id, issue_ordinal, created_at, updated_at))
            .into_boxed();
        match order.field {
            IssueField::SeriesId => match order.direction {
                Direction::Asc => query = query.order(series_id.asc()),
                Direction::Desc => query = query.order(series_id.desc()),
            },
            IssueField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            IssueField::IssueOrdinal => match order.direction {
                Direction::Asc => query = query.order(issue_ordinal.asc()),
                Direction::Desc => query = query.order(issue_ordinal.desc()),
            },
            IssueField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            IssueField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Issue>(&connection)
            .expect("Error loading issues")
    }

    #[graphql(description = "Query a single issue using its identifiers")]
    fn issue(context: &Context, series_id: Uuid, work_id: Uuid) -> FieldResult<Issue> {
        let connection = context.db.get().unwrap();
        crate::schema::issue::dsl::issue
            .filter(crate::schema::issue::dsl::series_id.eq(series_id))
            .filter(crate::schema::issue::dsl::work_id.eq(work_id))
            .get_result::<Issue>(&connection)
            .map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of issues")]
    fn issue_count(context: &Context) -> i32 {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        // see comment in work_count()
        issue
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading issue count")
            .to_string()
            .parse::<i32>()
            .unwrap()
    }

    #[graphql(
        description = "Query the full list of languages",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    LanguageOrderBy {
                        field: LanguageField::LanguageCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            language_code(description = "A specific language to filter by"),
            language_relation(description = "A specific relation to filter by"),
        )
    )]
    fn languages(
        context: &Context,
        limit: i32,
        offset: i32,
        order: LanguageOrderBy,
        publishers: Vec<Uuid>,
        language_code: Option<LanguageCode>,
        language_relation: Option<LanguageRelation>,
    ) -> FieldResult<Vec<Language>> {
        let connection = context.db.get().unwrap();
        Language::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            publishers,
            None,
            None,
            language_code,
            language_relation,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single language using its id")]
    fn language(context: &Context, language_id: Uuid) -> FieldResult<Language> {
        Language::from_id(&context.db, &language_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of languages associated to works")]
    fn language_count(
        context: &Context,
        language_code: Option<LanguageCode>,
        language_relation: Option<LanguageRelation>,
    ) -> FieldResult<i32> {
        Language::count(&context.db, None, vec![], language_code, language_relation)
            .map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of prices",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    PriceOrderBy {
                        field: PriceField::CurrencyCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            currency_code(description = "A specific currency to filter by"),
        )
    )]
    fn prices(
        context: &Context,
        limit: i32,
        offset: i32,
        order: PriceOrderBy,
        publishers: Vec<Uuid>,
        currency_code: Option<CurrencyCode>,
    ) -> FieldResult<Vec<Price>> {
        let connection = context.db.get().unwrap();
        Price::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            publishers,
            None,
            None,
            currency_code,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single price using its id")]
    fn price(context: &Context, price_id: Uuid) -> FieldResult<Price> {
        Price::from_id(&context.db, &price_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of prices associated to works")]
    fn price_count(context: &Context, currency_code: Option<CurrencyCode>) -> FieldResult<i32> {
        Price::count(&context.db, None, vec![], currency_code, None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of subjects",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on subject_code",
            ),
            order(
                default = {
                    SubjectOrderBy {
                        field: SubjectField::SubjectType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            subject_type(description = "A specific type to filter by"),
        )
    )]
    fn subjects(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: SubjectOrderBy,
        publishers: Vec<Uuid>,
        subject_type: Option<SubjectType>,
    ) -> FieldResult<Vec<Subject>> {
        Subject::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            subject_type,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single subject using its id")]
    fn subject(context: &Context, subject_id: Uuid) -> FieldResult<Subject> {
        Subject::from_id(&context.db, &subject_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of subjects associated to works")]
    fn subject_count(
        context: &Context,
        filter: String,
        subject_type: Option<SubjectType>,
    ) -> FieldResult<i32> {
        Subject::count(&context.db, Some(filter), vec![], subject_type, None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of funders",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on funderName and funderDoi",
            ),
            order(
                default = FunderOrderBy::default(),
                description = "The order in which to sort the results",
            ),
        )
    )]
    fn funders(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: FunderOrderBy,
    ) -> FieldResult<Vec<Funder>> {
        Funder::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            None,
            None,
            None,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single funder using its id")]
    fn funder(context: &Context, funder_id: Uuid) -> FieldResult<Funder> {
        Funder::from_id(&context.db, &funder_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of funders",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on funderName and funderDoi",
            ),
        )
    )]
    fn funder_count(context: &Context, filter: String) -> FieldResult<i32> {
        Funder::count(&context.db, Some(filter), vec![], None, None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of fundings",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    FundingOrderBy {
                        field: FundingField::Program,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn fundings(
        context: &Context,
        limit: i32,
        offset: i32,
        order: FundingOrderBy,
        publishers: Vec<Uuid>,
    ) -> FieldResult<Vec<Funding>> {
        Funding::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            None,
            None,
            None,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single funding using its id")]
    fn funding(context: &Context, funding_id: Uuid) -> FieldResult<Funding> {
        Funding::from_id(&context.db, &funding_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of funding instances associated to works")]
    fn funding_count(context: &Context) -> FieldResult<i32> {
        Funding::count(&context.db, None, vec![], None, None).map_err(|e| e.into())
    }
}

pub struct MutationRoot;

#[juniper::object(Context = Context)]
impl MutationRoot {
    fn create_work(context: &Context, data: NewWork) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_imprint(data.imprint_id, context)?;

        Work::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_publisher(context: &Context, data: NewPublisher) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        // Only superusers can create new publishers - NewPublisher has no ID field
        if !context.account_access.is_superuser {
            return Err(ThothError::Unauthorised.into());
        }

        Publisher::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_imprint(context: &Context, data: NewImprint) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(data.publisher_id)?;

        Imprint::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_contributor(context: &Context, data: NewContributor) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;

        Contributor::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_contribution(context: &Context, data: NewContribution) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();
        diesel::insert_into(contribution::table)
            .values(&data)
            .get_result(&connection)
            .map_err(|e| e.into())
    }

    fn create_publication(context: &Context, data: NewPublication) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        Publication::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_series(context: &Context, data: NewSeries) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_imprint(data.imprint_id, context)?;

        Series::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_issue(context: &Context, data: NewIssue) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;
        issue_imprints_match(data.work_id, data.series_id, context)?;

        let connection = context.db.get().unwrap();
        diesel::insert_into(issue::table)
            .values(&data)
            .get_result(&connection)
            .map_err(|e| e.into())
    }

    fn create_language(context: &Context, data: NewLanguage) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        Language::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_funder(context: &Context, data: NewFunder) -> FieldResult<Funder> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        Funder::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_funding(context: &Context, data: NewFunding) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        Funding::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_price(context: &Context, data: NewPrice) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_publication(data.publication_id, context)?;

        Price::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_subject(context: &Context, data: NewSubject) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        check_subject(&data.subject_type, &data.subject_code)?;

        Subject::create(&context.db, &data).map_err(|e| e.into())
    }

    fn update_work(context: &Context, data: PatchWork) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_imprint(data.imprint_id, context)?;

        let work = Work::from_id(&context.db, &data.work_id).unwrap();
        if !(data.imprint_id == work.imprint_id) {
            user_can_edit_imprint(work.imprint_id, context)?;
            can_update_work_imprint(work.work_id, context)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        work.update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_publisher(context: &Context, data: PatchPublisher) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(data.publisher_id)?;

        let publisher = Publisher::from_id(&context.db, &data.publisher_id).unwrap();
        if !(data.publisher_id == publisher.publisher_id) {
            context.account_access.can_edit(publisher.publisher_id)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        publisher
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_imprint(context: &Context, data: PatchImprint) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(data.publisher_id)?;

        let imprint = Imprint::from_id(&context.db, &data.imprint_id).unwrap();
        if !(data.publisher_id == imprint.publisher_id) {
            context.account_access.can_edit(imprint.publisher_id)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        imprint
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_contributor(context: &Context, data: PatchContributor) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        Contributor::from_id(&context.db, &data.contributor_id)
            .unwrap()
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_contribution(
        context: &Context,
        data: PatchContribution,
    ) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();

        use crate::schema::contribution::dsl::*;
        // need to duplicate these otherwise the query gets moved
        let target_contribution = contribution
            .filter(work_id.eq(&data.work_id))
            .filter(contributor_id.eq(&data.contributor_id))
            .filter(contribution_type.eq(&data.contribution_type))
            .get_result::<Contribution>(&connection)
            .unwrap();
        let target = contribution
            .filter(work_id.eq(&data.work_id))
            .filter(contributor_id.eq(&data.contributor_id))
            .filter(contribution_type.eq(&data.contribution_type));

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewContributionHistory::new(target_contribution, account_id)
                        .insert(&connection)
                    {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_publication(context: &Context, data: PatchPublication) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let publication = Publication::from_id(&context.db, &data.publication_id).unwrap();
        if !(data.work_id == publication.work_id) {
            user_can_edit_work(publication.work_id, context)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        publication
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_series(context: &Context, data: PatchSeries) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_imprint(data.imprint_id, context)?;

        let series = Series::from_id(&context.db, &data.series_id).unwrap();
        if !(data.imprint_id == series.imprint_id) {
            user_can_edit_imprint(series.imprint_id, context)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        series
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_issue(context: &Context, data: PatchIssue) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;
        issue_imprints_match(data.work_id, data.series_id, context)?;

        let connection = context.db.get().unwrap();

        use crate::schema::issue::dsl::*;
        let target = issue
            .filter(series_id.eq(&data.series_id))
            .filter(work_id.eq(&data.work_id));
        let target_issue = target.get_result::<Issue>(&connection).unwrap();

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewIssueHistory::new(target_issue, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_language(context: &Context, data: PatchLanguage) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let language = Language::from_id(&context.db, &data.language_id).unwrap();
        if !(data.work_id == language.work_id) {
            user_can_edit_work(language.work_id, context)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        language
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_funder(context: &Context, data: PatchFunder) -> FieldResult<Funder> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        Funder::from_id(&context.db, &data.funder_id)
            .unwrap()
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_funding(context: &Context, data: PatchFunding) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let funding = Funding::from_id(&context.db, &data.funding_id).unwrap();
        if !(data.work_id == funding.work_id) {
            user_can_edit_work(funding.work_id, context)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        funding
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_price(context: &Context, data: PatchPrice) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_publication(data.publication_id, context)?;

        let price = Price::from_id(&context.db, &data.price_id).unwrap();
        if !(data.publication_id == price.publication_id) {
            user_can_edit_publication(price.publication_id, context)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        price
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_subject(context: &Context, data: PatchSubject) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let subject = Subject::from_id(&context.db, &data.subject_id).unwrap();
        if !(data.work_id == subject.work_id) {
            user_can_edit_work(subject.work_id, context)?;
        }

        check_subject(&data.subject_type, &data.subject_code)?;

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        subject
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn delete_work(context: &Context, work_id: Uuid) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(work_id, context)?;

        Work::from_id(&context.db, &work_id)
            .unwrap()
            .delete(&context.db)
            .map_err(|e| e.into())
    }

    fn delete_publisher(context: &Context, publisher_id: Uuid) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(publisher_id)?;

        Publisher::from_id(&context.db, &publisher_id)
            .unwrap()
            .delete(&context.db)
            .map_err(|e| e.into())
    }

    fn delete_imprint(context: &Context, imprint_id: Uuid) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let imprint = Imprint::from_id(&context.db, &imprint_id).unwrap();
        context.account_access.can_edit(imprint.publisher_id)?;

        imprint.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_contributor(context: &Context, contributor_id: Uuid) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        Contributor::from_id(&context.db, &contributor_id)
            .unwrap()
            .delete(&context.db)
            .map_err(|e| e.into())
    }

    fn delete_contribution(
        context: &Context,
        work_id: Uuid,
        contributor_id: Uuid,
        contribution_type: ContributionType,
    ) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(work_id, context)?;

        let connection = context.db.get().unwrap();

        use crate::schema::contribution::dsl;
        let target = dsl::contribution
            .filter(dsl::work_id.eq(&work_id))
            .filter(dsl::contributor_id.eq(&contributor_id))
            .filter(dsl::contribution_type.eq(&contribution_type));
        let result = dsl::contribution
            .filter(dsl::work_id.eq(&work_id))
            .filter(dsl::contributor_id.eq(&contributor_id))
            .filter(dsl::contribution_type.eq(&contribution_type))
            .get_result::<Contribution>(&connection);
        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(result.unwrap()),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_publication(context: &Context, publication_id: Uuid) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_publication(publication_id, context)?;

        Publication::from_id(&context.db, &publication_id)
            .unwrap()
            .delete(&context.db)
            .map_err(|e| e.into())
    }

    fn delete_series(context: &Context, series_id: Uuid) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let series = Series::from_id(&context.db, &series_id).unwrap();
        user_can_edit_imprint(series.imprint_id, context)?;

        series.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_issue(context: &Context, series_id: Uuid, work_id: Uuid) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(work_id, context)?;

        let connection = context.db.get().unwrap();

        use crate::schema::issue::dsl;
        let target = dsl::issue
            .filter(dsl::series_id.eq(&series_id))
            .filter(dsl::work_id.eq(&work_id));
        let result = dsl::issue
            .filter(dsl::series_id.eq(&series_id))
            .filter(dsl::work_id.eq(&work_id))
            .get_result::<Issue>(&connection);
        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(result.unwrap()),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_language(context: &Context, language_id: Uuid) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let language = Language::from_id(&context.db, &language_id).unwrap();
        user_can_edit_work(language.work_id, context)?;

        language.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_funder(context: &Context, funder_id: Uuid) -> FieldResult<Funder> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        Funder::from_id(&context.db, &funder_id)
            .unwrap()
            .delete(&context.db)
            .map_err(|e| e.into())
    }

    fn delete_funding(context: &Context, funding_id: Uuid) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let funding = Funding::from_id(&context.db, &funding_id).unwrap();
        user_can_edit_work(funding.work_id, context)?;

        funding.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_price(context: &Context, price_id: Uuid) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let price = Price::from_id(&context.db, &price_id).unwrap();
        user_can_edit_publication(price.publication_id, context)?;

        price.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_subject(context: &Context, subject_id: Uuid) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let subject = Subject::from_id(&context.db, &subject_id).unwrap();
        user_can_edit_work(subject.work_id, context)?;

        subject.delete(&context.db).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A written text that can be published")]
impl Work {
    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn work_type(&self) -> &WorkType {
        &self.work_type
    }

    pub fn work_status(&self) -> &WorkStatus {
        &self.work_status
    }

    #[graphql(description = "Concatenation of title and subtitle with punctuation mark")]
    pub fn full_title(&self) -> &str {
        self.full_title.as_str()
    }

    #[graphql(description = "Main title of the work (excluding subtitle)")]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    #[graphql(description = "Secondary title of the work (excluding main title)")]
    pub fn subtitle(&self) -> Option<&String> {
        self.subtitle.as_ref()
    }

    #[graphql(description = "Internal reference code")]
    pub fn reference(&self) -> Option<&String> {
        self.reference.as_ref()
    }

    pub fn edition(&self) -> &i32 {
        &self.edition
    }

    #[graphql(
        description = "Digital Object Identifier of the work as full URL. It must use the HTTPS scheme and the doi.org domain (e.g. https://doi.org/10.11647/obp.0001)"
    )]
    pub fn doi(&self) -> Option<&String> {
        self.doi.as_ref()
    }

    pub fn publication_date(&self) -> Option<NaiveDate> {
        self.publication_date
    }

    pub fn place(&self) -> Option<&String> {
        self.place.as_ref()
    }

    pub fn width(&self) -> Option<&i32> {
        self.width.as_ref()
    }

    pub fn height(&self) -> Option<&i32> {
        self.height.as_ref()
    }

    pub fn page_count(&self) -> Option<&i32> {
        self.page_count.as_ref()
    }

    pub fn page_breakdown(&self) -> Option<&String> {
        self.page_breakdown.as_ref()
    }

    pub fn image_count(&self) -> Option<&i32> {
        self.image_count.as_ref()
    }

    pub fn table_count(&self) -> Option<&i32> {
        self.table_count.as_ref()
    }

    pub fn audio_count(&self) -> Option<&i32> {
        self.audio_count.as_ref()
    }

    pub fn video_count(&self) -> Option<&i32> {
        self.video_count.as_ref()
    }

    pub fn license(&self) -> Option<&String> {
        self.license.as_ref()
    }

    pub fn copyright_holder(&self) -> &str {
        self.copyright_holder.as_str()
    }

    pub fn landing_page(&self) -> Option<&String> {
        self.landing_page.as_ref()
    }

    pub fn lccn(&self) -> Option<&String> {
        self.lccn.as_ref()
    }

    pub fn oclc(&self) -> Option<&String> {
        self.oclc.as_ref()
    }

    pub fn short_abstract(&self) -> Option<&String> {
        self.short_abstract.as_ref()
    }

    pub fn long_abstract(&self) -> Option<&String> {
        self.long_abstract.as_ref()
    }

    pub fn general_note(&self) -> Option<&String> {
        self.general_note.as_ref()
    }

    pub fn toc(&self) -> Option<&String> {
        self.toc.as_ref()
    }

    pub fn cover_url(&self) -> Option<&String> {
        self.cover_url.as_ref()
    }

    pub fn cover_caption(&self) -> Option<&String> {
        self.cover_caption.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn imprint(&self, context: &Context) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &self.imprint_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get contributions linked to this work",
        arguments(
            order(
                default = {
                    ContributionOrderBy {
                        field: ContributionField::ContributionType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            contribution_type(description = "A specific type to filter by"),
        )
    )]
    pub fn contributions(
        &self,
        context: &Context,
        order: ContributionOrderBy,
        contribution_type: Option<ContributionType>,
    ) -> Vec<Contribution> {
        use crate::schema::contribution::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::contribution.into_boxed();
        match order.field {
            ContributionField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            ContributionField::ContributorId => match order.direction {
                Direction::Asc => query = query.order(dsl::contributor_id.asc()),
                Direction::Desc => query = query.order(dsl::contributor_id.desc()),
            },
            ContributionField::ContributionType => match order.direction {
                Direction::Asc => query = query.order(dsl::contribution_type.asc()),
                Direction::Desc => query = query.order(dsl::contribution_type.desc()),
            },
            ContributionField::MainContribution => match order.direction {
                Direction::Asc => query = query.order(dsl::main_contribution.asc()),
                Direction::Desc => query = query.order(dsl::main_contribution.desc()),
            },
            ContributionField::Biography => match order.direction {
                Direction::Asc => query = query.order(dsl::biography.asc()),
                Direction::Desc => query = query.order(dsl::biography.desc()),
            },
            ContributionField::Institution => match order.direction {
                Direction::Asc => query = query.order(dsl::institution.asc()),
                Direction::Desc => query = query.order(dsl::institution.desc()),
            },
            ContributionField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            ContributionField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
            ContributionField::FirstName => match order.direction {
                Direction::Asc => query = query.order(dsl::first_name.asc()),
                Direction::Desc => query = query.order(dsl::first_name.desc()),
            },
            ContributionField::LastName => match order.direction {
                Direction::Asc => query = query.order(dsl::last_name.asc()),
                Direction::Desc => query = query.order(dsl::last_name.desc()),
            },
            ContributionField::FullName => match order.direction {
                Direction::Asc => query = query.order(dsl::full_name.asc()),
                Direction::Desc => query = query.order(dsl::full_name.desc()),
            },
        }
        if let Some(cont_type) = contribution_type {
            query = query.filter(dsl::contribution_type.eq(cont_type))
        }
        query
            .filter(dsl::work_id.eq(self.work_id))
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }

    #[graphql(
        description = "Get languages linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    LanguageOrderBy {
                        field: LanguageField::LanguageCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            language_code(description = "A specific language to filter by"),
            language_relation(description = "A specific relation to filter by"),
        )
    )]
    pub fn languages(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: LanguageOrderBy,
        language_code: Option<LanguageCode>,
        language_relation: Option<LanguageRelation>,
    ) -> FieldResult<Vec<Language>> {
        Language::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.work_id),
            None,
            language_code,
            language_relation,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get publications linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn and publication_url"
            ),
            order(
                default = {
                    PublicationOrderBy {
                        field: PublicationField::PublicationType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publication_type(description = "A specific type to filter by"),
        )
    )]
    pub fn publications(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: PublicationOrderBy,
        publication_type: Option<PublicationType>,
    ) -> FieldResult<Vec<Publication>> {
        Publication::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            Some(self.work_id),
            None,
            publication_type,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get subjects linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on subject_code",
            ),
            order(
                default = {
                    SubjectOrderBy {
                        field: SubjectField::SubjectType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            subject_type(description = "A specific type to filter by"),
        )
    )]
    pub fn subjects(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: SubjectOrderBy,
        subject_type: Option<SubjectType>,
    ) -> FieldResult<Vec<Subject>> {
        Subject::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            Some(self.work_id),
            None,
            subject_type,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get fundings linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    FundingOrderBy {
                        field: FundingField::Program,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn fundings(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: FundingOrderBy,
    ) -> FieldResult<Vec<Funding>> {
        Funding::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.work_id),
            None,
            None,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get issues linked to this work",
        arguments(
            order(
                default = {
                    IssueOrderBy {
                        field: IssueField::IssueOrdinal,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn issues(&self, context: &Context, order: IssueOrderBy) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = issue.into_boxed();
        match order.field {
            IssueField::SeriesId => match order.direction {
                Direction::Asc => query = query.order(series_id.asc()),
                Direction::Desc => query = query.order(series_id.desc()),
            },
            IssueField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            IssueField::IssueOrdinal => match order.direction {
                Direction::Asc => query = query.order(issue_ordinal.asc()),
                Direction::Desc => query = query.order(issue_ordinal.desc()),
            },
            IssueField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            IssueField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        query
            .filter(work_id.eq(self.work_id))
            .load::<Issue>(&connection)
            .expect("Error loading issues")
    }
}

#[juniper::object(Context = Context, description = "A manifestation of a written text")]
impl Publication {
    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    pub fn publication_type(&self) -> &PublicationType {
        &self.publication_type
    }

    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn isbn(&self) -> Option<&String> {
        self.isbn.as_ref()
    }

    pub fn publication_url(&self) -> Option<&String> {
        self.publication_url.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(
        description = "Get prices linked to this publication",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    PriceOrderBy {
                        field: PriceField::CurrencyCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            currency_code(description = "A specific currency to filter by"),
        )
    )]
    pub fn prices(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: PriceOrderBy,
        currency_code: Option<CurrencyCode>,
    ) -> FieldResult<Vec<Price>> {
        Price::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.publication_id),
            None,
            currency_code,
            None,
        )
        .map_err(|e| e.into())
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "An organisation that produces and distributes written texts.")]
impl Publisher {
    pub fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }

    pub fn publisher_name(&self) -> &String {
        &self.publisher_name
    }

    pub fn publisher_shortname(&self) -> Option<&String> {
        self.publisher_shortname.as_ref()
    }

    pub fn publisher_url(&self) -> Option<&String> {
        self.publisher_url.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(
        description = "Get imprints linked to this publisher",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on imprint_name and imprint_url"
            ),
            order(
                default = {
                    ImprintOrderBy {
                        field: ImprintField::ImprintName,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn imprints(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: ImprintOrderBy,
    ) -> FieldResult<Vec<Imprint>> {
        Imprint::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            Some(self.publisher_id),
            None,
            None,
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "The brand under which a publisher issues works.")]
impl Imprint {
    pub fn imprint_id(&self) -> Uuid {
        self.imprint_id
    }

    pub fn imprint_name(&self) -> &String {
        &self.imprint_name
    }

    pub fn imprint_url(&self) -> Option<&String> {
        self.imprint_url.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn publisher(&self, context: &Context) -> FieldResult<Publisher> {
        Publisher::from_id(&context.db, &self.publisher_id).map_err(|e| e.into())
    }

    #[graphql(
    description="Get works linked to this imprint",
    arguments(
        limit(
            default = 100,
            description = "The number of items to return"
        ),
        offset(
            default = 0,
            description = "The number of items to skip"
        ),
        filter(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        ),
        order(
            default = {
                WorkOrderBy {
                    field: WorkField::FullTitle,
                    direction: Direction::Asc,
                }
            },
            description = "The order in which to sort the results",
        ),
        work_type(description = "A specific type to filter by"),
        work_status(description = "A specific status to filter by"),
    )
  )]
    pub fn works(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: WorkOrderBy,
        work_type: Option<WorkType>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<Vec<Work>> {
        Work::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            Some(self.imprint_id),
            None,
            work_type,
            work_status,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A person who has been involved in the production of a written text.")]
impl Contributor {
    pub fn contributor_id(&self) -> Uuid {
        self.contributor_id
    }

    pub fn first_name(&self) -> Option<&String> {
        self.first_name.as_ref()
    }

    pub fn last_name(&self) -> &String {
        &self.last_name
    }

    pub fn full_name(&self) -> &String {
        &self.full_name
    }

    pub fn orcid(&self) -> Option<&String> {
        self.orcid.as_ref()
    }

    pub fn website(&self) -> Option<&String> {
        self.website.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(
        description = "Get contributions linked to this contributor",
        arguments(
            order(
                default = {
                    ContributionOrderBy {
                        field: ContributionField::ContributionType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            contribution_type(description = "A specific type to filter by"),
        )
    )]
    pub fn contributions(
        &self,
        context: &Context,
        order: ContributionOrderBy,
        contribution_type: Option<ContributionType>,
    ) -> Vec<Contribution> {
        use crate::schema::contribution::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::contribution.into_boxed();
        match order.field {
            ContributionField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            ContributionField::ContributorId => match order.direction {
                Direction::Asc => query = query.order(dsl::contributor_id.asc()),
                Direction::Desc => query = query.order(dsl::contributor_id.desc()),
            },
            ContributionField::ContributionType => match order.direction {
                Direction::Asc => query = query.order(dsl::contribution_type.asc()),
                Direction::Desc => query = query.order(dsl::contribution_type.desc()),
            },
            ContributionField::MainContribution => match order.direction {
                Direction::Asc => query = query.order(dsl::main_contribution.asc()),
                Direction::Desc => query = query.order(dsl::main_contribution.desc()),
            },
            ContributionField::Biography => match order.direction {
                Direction::Asc => query = query.order(dsl::biography.asc()),
                Direction::Desc => query = query.order(dsl::biography.desc()),
            },
            ContributionField::Institution => match order.direction {
                Direction::Asc => query = query.order(dsl::institution.asc()),
                Direction::Desc => query = query.order(dsl::institution.desc()),
            },
            ContributionField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            ContributionField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
            ContributionField::FirstName => match order.direction {
                Direction::Asc => query = query.order(dsl::first_name.asc()),
                Direction::Desc => query = query.order(dsl::first_name.desc()),
            },
            ContributionField::LastName => match order.direction {
                Direction::Asc => query = query.order(dsl::last_name.asc()),
                Direction::Desc => query = query.order(dsl::last_name.desc()),
            },
            ContributionField::FullName => match order.direction {
                Direction::Asc => query = query.order(dsl::full_name.asc()),
                Direction::Desc => query = query.order(dsl::full_name.desc()),
            },
        }
        if let Some(cont_type) = contribution_type {
            query = query.filter(dsl::contribution_type.eq(cont_type))
        }
        query
            .filter(dsl::contributor_id.eq(self.contributor_id))
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }
}

#[juniper::object(Context = Context, description = "A person's involvement in the production of a written text.")]
impl Contribution {
    pub fn contributor_id(&self) -> Uuid {
        self.contributor_id
    }

    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn contribution_type(&self) -> &ContributionType {
        &self.contribution_type
    }

    pub fn main_contribution(&self) -> bool {
        self.main_contribution
    }

    pub fn biography(&self) -> Option<&String> {
        self.biography.as_ref()
    }

    pub fn institution(&self) -> Option<&String> {
        self.institution.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn first_name(&self) -> Option<&String> {
        self.first_name.as_ref()
    }

    pub fn last_name(&self) -> &String {
        &self.last_name
    }

    pub fn full_name(&self) -> &String {
        &self.full_name
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }

    pub fn contributor(&self, context: &Context) -> FieldResult<Contributor> {
        Contributor::from_id(&context.db, &self.contributor_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A periodical of publications about a particular subject.")]
impl Series {
    pub fn series_id(&self) -> Uuid {
        self.series_id
    }

    pub fn series_type(&self) -> &SeriesType {
        &self.series_type
    }

    pub fn series_name(&self) -> &String {
        &self.series_name
    }

    pub fn issn_print(&self) -> &String {
        &self.issn_print
    }

    pub fn issn_digital(&self) -> &String {
        &self.issn_digital
    }

    pub fn series_url(&self) -> Option<&String> {
        self.series_url.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    //see comments on similar fn above
    pub fn imprint(&self, context: &Context) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &self.imprint_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get issues linked to this series",
        arguments(
            order(
                default = {
                    IssueOrderBy {
                        field: IssueField::IssueOrdinal,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn issues(&self, context: &Context, order: IssueOrderBy) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = issue.into_boxed();
        match order.field {
            IssueField::SeriesId => match order.direction {
                Direction::Asc => query = query.order(series_id.asc()),
                Direction::Desc => query = query.order(series_id.desc()),
            },
            IssueField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            IssueField::IssueOrdinal => match order.direction {
                Direction::Asc => query = query.order(issue_ordinal.asc()),
                Direction::Desc => query = query.order(issue_ordinal.desc()),
            },
            IssueField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            IssueField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        query
            .filter(series_id.eq(self.series_id))
            .load::<Issue>(&connection)
            .expect("Error loading issues")
    }
}

#[juniper::object(Context = Context, description = "A work published as a number in a periodical.")]
impl Issue {
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn series_id(&self) -> Uuid {
        self.series_id
    }

    pub fn issue_ordinal(&self) -> &i32 {
        &self.issue_ordinal
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn series(&self, context: &Context) -> FieldResult<Series> {
        Series::from_id(&context.db, &self.series_id).map_err(|e| e.into())
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "Description of a work's language.")]
impl Language {
    pub fn language_id(&self) -> Uuid {
        self.language_id
    }

    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn language_code(&self) -> &LanguageCode {
        &self.language_code
    }

    pub fn language_relation(&self) -> &LanguageRelation {
        &self.language_relation
    }

    pub fn main_language(&self) -> bool {
        self.main_language
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "The amount of money, in any currency, that a publication costs.")]
impl Price {
    pub fn price_id(&self) -> Uuid {
        self.price_id
    }

    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    pub fn currency_code(&self) -> &CurrencyCode {
        &self.currency_code
    }

    pub fn unit_price(&self) -> f64 {
        self.unit_price
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn publication(&self, context: &Context) -> FieldResult<Publication> {
        Publication::from_id(&context.db, &self.publication_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A significant discipline or term related to a work.")]
impl Subject {
    pub fn subject_id(&self) -> &Uuid {
        &self.subject_id
    }

    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn subject_type(&self) -> &SubjectType {
        &self.subject_type
    }

    pub fn subject_code(&self) -> &String {
        &self.subject_code
    }

    pub fn subject_ordinal(&self) -> &i32 {
        &self.subject_ordinal
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "An organisation that provides the money to pay for the publication of a work.")]
impl Funder {
    pub fn funder_id(&self) -> &Uuid {
        &self.funder_id
    }

    pub fn funder_name(&self) -> &String {
        &self.funder_name
    }

    pub fn funder_doi(&self) -> Option<&String> {
        self.funder_doi.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(
        description = "Get fundings linked to this funder",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    FundingOrderBy {
                        field: FundingField::Program,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn fundings(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: FundingOrderBy,
    ) -> FieldResult<Vec<Funding>> {
        Funding::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            None,
            Some(self.funder_id),
            None,
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A grant awarded to the publication of a work by a funder.")]
impl Funding {
    pub fn funding_id(&self) -> &Uuid {
        &self.funding_id
    }

    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn funder_id(&self) -> &Uuid {
        &self.funder_id
    }

    pub fn program(&self) -> Option<&String> {
        self.program.as_ref()
    }

    pub fn project_name(&self) -> Option<&String> {
        self.project_name.as_ref()
    }

    pub fn project_shortname(&self) -> Option<&String> {
        self.project_shortname.as_ref()
    }

    pub fn grant_number(&self) -> Option<&String> {
        self.grant_number.as_ref()
    }

    pub fn jurisdiction(&self) -> Option<&String> {
        self.jurisdiction.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }

    pub fn funder(&self, context: &Context) -> FieldResult<Funder> {
        Funder::from_id(&context.db, &self.funder_id).map_err(|e| e.into())
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

fn user_can_edit_imprint(imprint_id: Uuid, context: &Context) -> ThothResult<()> {
    use crate::schema::imprint::dsl;
    let pub_id = dsl::imprint
        .select(dsl::publisher_id)
        .filter(dsl::imprint_id.eq(imprint_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error checking permissions");
    context.account_access.can_edit(pub_id)
}

fn user_can_edit_work(work_id: Uuid, context: &Context) -> ThothResult<()> {
    use crate::schema::imprint::dsl::*;
    let pub_id = imprint
        .inner_join(crate::schema::work::table)
        .select(publisher_id)
        .filter(crate::schema::work::work_id.eq(work_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error checking permissions");
    context.account_access.can_edit(pub_id)
}

fn user_can_edit_publication(publication_id: Uuid, context: &Context) -> ThothResult<()> {
    use crate::schema::imprint::dsl::*;
    let pub_id = imprint
        .inner_join(crate::schema::work::table.inner_join(crate::schema::publication::table))
        .select(publisher_id)
        .filter(crate::schema::publication::publication_id.eq(publication_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error checking permissions");
    context.account_access.can_edit(pub_id)
}

fn issue_imprints_match(work_id: Uuid, series_id: Uuid, context: &Context) -> ThothResult<()> {
    let series_imprint = crate::schema::series::table
        .select(crate::schema::series::imprint_id)
        .filter(crate::schema::series::series_id.eq(series_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error loading series for issue");
    let work_imprint = crate::schema::work::table
        .select(crate::schema::work::imprint_id)
        .filter(crate::schema::work::work_id.eq(work_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error loading work for issue");
    if work_imprint == series_imprint {
        Ok(())
    } else {
        Err(ThothError::IssueImprintsError)
    }
}

fn can_update_work_imprint(work_id: Uuid, context: &Context) -> ThothResult<()> {
    use crate::schema::issue::dsl;
    // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
    // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
    // is converting i64 to string and then parsing it as i32. This should work until we reach
    // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
    let issue_count = dsl::issue
        .filter(dsl::work_id.eq(work_id))
        .count()
        .get_result::<i64>(&context.db.get().unwrap())
        .expect("Error loading issue count for work")
        .to_string()
        .parse::<i32>()
        .unwrap();
    // If a work has any related issues, its imprint cannot be changed,
    // because an issue's series and work must both have the same imprint.
    if issue_count == 0 {
        Ok(())
    } else {
        Err(ThothError::IssueImprintsError)
    }
}

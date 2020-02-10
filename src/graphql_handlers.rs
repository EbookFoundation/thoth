extern crate dotenv;

use diesel::prelude::*;
use juniper::RootNode;
use uuid::Uuid;
use chrono::naive::NaiveDate;

use crate::db::PgPool;
use crate::schema::work;

use crate::models::publisher::*;
use crate::models::work::*;
use crate::models::language::*;
use crate::models::series::*;
use crate::models::contributor::*;
use crate::models::publication::*;
use crate::models::price::*;
use crate::models::keyword::*;

#[derive(Clone)]
pub struct Context {
  pub db: PgPool,
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::object(Context = Context)]
impl QueryRoot {
  fn works(context: &Context) -> Vec<Work> {
    use crate::schema::work::dsl::*;
    let connection = context.db.get().unwrap();
    work
      .limit(100)
      .load::<Work>(&connection)
      .expect("Error loading works")
  }

  fn publications(context: &Context) -> Vec<Publication> {
    use crate::schema::publication::dsl::*;
    let connection = context.db.get().unwrap();
    publication
      .limit(100)
      .load::<Publication>(&connection)
      .expect("Error loading publications")
  }

  fn publishers(context: &Context) -> Vec<Publisher> {
    use crate::schema::publisher::dsl::*;
    let connection = context.db.get().unwrap();
    publisher
      .limit(100)
      .load::<Publisher>(&connection)
      .expect("Error loading publishers")
  }

  fn contributors(context: &Context) -> Vec<Contributor> {
    use crate::schema::contributor::dsl::*;
    let connection = context.db.get().unwrap();
    contributor
        .limit(100)
        .load::<Contributor>(&connection)
        .expect("Error loading contributors")
  }

  fn series(context: &Context) -> Vec<Series> {
    use crate::schema::series::dsl::*;
    let connection = context.db.get().unwrap();
    series
        .limit(100)
        .load::<Series>(&connection)
        .expect("Error loading series")
  }

  fn issues(context: &Context) -> Vec<Issue> {
    use crate::schema::issue::dsl::*;
    let connection = context.db.get().unwrap();
    issue
        .limit(100)
        .load::<Issue>(&connection)
        .expect("Error loading issues")
  }

  fn languages(context: &Context) -> Vec<Language> {
    use crate::schema::language::dsl::*;
    let connection = context.db.get().unwrap();
    language
        .limit(100)
        .load::<Language>(&connection)
        .expect("Error loading languages")
  }

  fn prices(context: &Context) -> Vec<Price> {
    use crate::schema::price::dsl::*;
    let connection = context.db.get().unwrap();
    price
        .limit(100)
        .load::<Price>(&connection)
        .expect("Error loading prices")
  }

  fn keywords(context: &Context) -> Vec<Keyword> {
    use crate::schema::keyword::dsl::*;
    let connection = context.db.get().unwrap();
    keyword
        .limit(100)
        .load::<Keyword>(&connection)
        .expect("Error loading keyword")
  }
}

pub struct MutationRoot;

#[juniper::object(Context = Context)]
impl MutationRoot {
  fn create_work(context: &Context, data: NewWork) -> Work {
    let connection = context.db.get().unwrap();
    diesel::insert_into(work::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new work")
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

    #[graphql(description="Concatenation of title and subtitle with punctuation mark")]
    pub fn full_title(&self) -> &str {
        self.full_title.as_str()
    }

    #[graphql(description="Main title of the work (excluding subtitle)")]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    #[graphql(description="Secondary title of the work (excluding main title)")]
    pub fn subtitle(&self) -> Option<&String> {
        self.subtitle.as_ref()
    }

    #[graphql(description="Internal reference code")]
    pub fn reference(&self) -> Option<&String> {
        self.reference.as_ref()
    }

    pub fn edition(&self) -> &i32 {
        &self.edition
    }

    #[graphql(description="Digital Object Identifier of the work as full URL. It must use the HTTPS scheme and the doi.org domain (e.g. https://doi.org/10.11647/obp.0001)")]
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

    pub fn lccn(&self) -> Option<&i32> {
        self.lccn.as_ref()
    }

    pub fn oclc(&self) -> Option<&i32> {
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

    pub fn publisher(&self, context: &Context) -> Publisher {
        use crate::schema::publisher::dsl::*;
        let connection = context.db.get().unwrap();
        publisher
            .find(self.publisher_id)
            .first(&connection)
            .expect("Error loading publisher")
    }

    pub fn contributions(&self, context: &Context) -> Vec<Contribution> {
        use crate::schema::contribution::dsl::*;
        let connection = context.db.get().unwrap();
        contribution
            .filter(work_id.eq(self.work_id))
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }

    pub fn languages(&self, context: &Context) -> Vec<Language> {
        use crate::schema::language::dsl::*;
        let connection = context.db.get().unwrap();
        language
            .filter(work_id.eq(self.work_id))
            .load::<Language>(&connection)
            .expect("Error loading languages")
    }

    pub fn publications(&self, context: &Context) -> Vec<Publication> {
        use crate::schema::publication::dsl::*;
        let connection = context.db.get().unwrap();
        publication
            .filter(work_id.eq(self.work_id))
            .load::<Publication>(&connection)
            .expect("Error loading publications")
    }

    pub fn keywords(&self, context: &Context) -> Vec<Keyword> {
        use crate::schema::keyword::dsl::*;
        let connection = context.db.get().unwrap();
        keyword
            .filter(work_id.eq(self.work_id))
            .load::<Keyword>(&connection)
            .expect("Error loading keywords")
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

    pub fn isbn(&self) -> Option<&String> {
        self.isbn.as_ref()
    }

    pub fn publication_url(&self) -> Option<&String> {
        self.publication_url.as_ref()
    }

    pub fn prices(&self, context: &Context) -> Vec<Price> {
        use crate::schema::price::dsl::*;
        let connection = context.db.get().unwrap();
        price
            .filter(publication_id.eq(self.publication_id))
            .load::<Price>(&connection)
            .expect("Error loading price")
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }
}

#[juniper::object(description = "An organisation that produces and distributes written texts.")]
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

    pub fn contributions(&self, context: &Context) -> Vec<Contribution> {
        use crate::schema::contribution::dsl::*;
        let connection = context.db.get().unwrap();
        contribution
            .filter(contributor_id.eq(self.contributor_id))
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

    pub fn main_contribution(&self) -> bool{
        self.main_contribution
    }

    pub fn biography(&self) -> Option<&String> {
        self.biography.as_ref()
    }

    pub fn institution(&self) -> Option<&String> {
        self.institution.as_ref()
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }

    pub fn contributor(&self, context: &Context) -> Contributor {
        use crate::schema::contributor::dsl::*;
        let connection = context.db.get().unwrap();
        contributor
            .find(self.contributor_id)
            .first(&connection)
            .expect("Error loading contributions")
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

    pub fn publisher(&self, context: &Context) -> Publisher {
        use crate::schema::publisher::dsl::*;
        let connection = context.db.get().unwrap();
        publisher
            .find(self.publisher_id)
            .first(&connection)
            .expect("Error loading publisher")
    }

    pub fn issues(&self, context: &Context) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        issue
            .filter(series_id.eq(self.series_id))
            .load::<Issue>(&connection)
            .expect("Error loading issues")
    }
}

#[juniper::object(Context = Context, description = "A work published as a number in a periodical.")]
impl Issue {
    pub fn issue_ordinal(&self) -> &i32 {
        &self.issue_ordinal
    }

    pub fn series(&self, context: &Context) -> Series {
        use crate::schema::series::dsl::*;
        let connection = context.db.get().unwrap();
        series
            .find(self.series_id)
            .first(&connection)
            .expect("Error loading series")
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }
}

#[juniper::object(Context = Context, description = "Description of a work's language.")]
impl Language {
    pub fn language_id(&self) -> Uuid {
        self.language_id
    }

    pub fn language_code(&self) -> &LanguageCode {
        &self.language_code
    }

    pub fn language_relation(&self) -> &LanguageRelation {
        &self.language_relation
    }

    pub fn main_language(&self) -> bool{
        self.main_language
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }
}

#[juniper::object(Context = Context, description = "The amount of money, in any currency, that a publication costs.")]
impl Price {
    pub fn price_id(&self) -> Uuid {
        self.price_id
    }

    pub fn currency_code(&self) -> &CurrencyCode {
        &self.currencty_code
    }

    pub fn unit_price(&self) -> f64 {
        self.unit_price
    }

    pub fn publication(&self, context: &Context) -> Publication {
        use crate::schema::publication::dsl::*;
        let connection = context.db.get().unwrap();
        publication
            .find(self.publication_id)
            .first(&connection)
            .expect("Error loading publication")
    }
}

#[juniper::object(Context = Context, description = "A significant term related to a work.")]
impl Keyword {
    pub fn keyword_term(&self) -> &String {
        &self.keyword_term
    }

    pub fn keyword_ordinal(&self) -> &i32 {
        &self.keyword_ordinal
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
  Schema::new(QueryRoot {}, MutationRoot {})
}

use phf::phf_map;
use phf::Map;

use crate::ThothError;

/// A map of database constraint name and a corresponding error to output
/// when the constraint is violated.
///
/// To obtain a list of unique and check constraints:
/// ```sql
/// SELECT conname
/// FROM pg_catalog.pg_constraint con
/// INNER JOIN pg_catalog.pg_namespace nsp ON nsp.oid = connamespace
/// WHERE nsp.nspname = 'public'
/// AND contype in ('u', 'c');
/// ```
static DATABASE_CONSTRAINT_ERRORS: Map<&'static str, &'static str> = phf_map! {
    "publisher_uniq_idx" => "A publisher with this name already exists.",
    "imprint_uniq_idx" => "An imprint with this name already exists.",
    "doi_uniq_idx" => "A work with this DOI already exists.",
    "language_uniq_work_idx" => "Duplicate language code.",
    "series_issn_print_idx" => "A series with this ISSN already exists.",
    "series_issn_digital_idx" => "A series with this ISSN already exists.",
    "issue_uniq_ord_in_series_idx" => "An issue with this ordinal number already exists.",
    "orcid_uniq_idx" => "A contributor with this ORCID ID already exists.",
    "location_uniq_canonical_true_idx" => "A canonical location for this publication already exists.",
    "location_uniq_platform_idx" => "A location on the selected platform already exists.",
    "email_uniq_idx" => "An account with this email already exists.",
    "affiliation_uniq_ord_in_contribution_idx" => "An affiliation with this ordinal number already exists.",
    "contribution_contribution_ordinal_work_id_uniq" => "A contribution with this ordinal number already exists.",
    "contribution_work_id_contributor_id_contribution_type_uniq" => "A contribution of this type already exists for this contributor.",
    "issue_series_id_work_id_uniq" => "An issue on the selected series already exists for the this work.",
    "publication_publication_type_work_id_uniq" => "A publication with the selected type already exists.",
    "work_relation_ordinal_type_uniq" => "A relation with this ordinal number already exists.",
    "work_relation_relator_related_uniq" => "A relation between these two works already exists.",
    "affiliation_affiliation_ordinal_check" => "An affiliation ordinal number must be greater than 0.",
    "contribution_contribution_ordinal_check" => "A contribution ordinal number must be greater than 0.",
    "affiliation_position_check" => "Position must not be an empty string.",
    "contribution_biography_check" => "Biography must not be an empty string.",
    "contribution_first_name_check" => "First name must not be an empty string.",
    "contribution_full_name_check" => "Full name must not be an empty string.",
    "contribution_last_name_check" => "Last name must not be an empty string.",
    "contributor_first_name_check" => "First name must not be an empty string.",
    "contributor_full_name_check" => "Full name must not be an empty string.",
    "contributor_last_name_check" => "Last name must not be an empty string.",
    "contributor_orcid_check" => "Invalid ORCID ID.",
    "contributor_website_check" => "Website must not be an empty string.",
    "funding_grant_number_check" => "Grant number must not be an empty string.",
    "funding_jurisdiction_check" => "Jurisdiction must not be an empty string.",
    "funding_program_check" => "Program must not be an empty string.",
    "funding_project_name_check" => "Project name must not be an empty string.",
    "funding_project_shortname_check" => "Project shortname must not be an empty string.",
    "imprint_imprint_name_check" => "Imprint name must not be an empty string.",
    "imprint_imprint_url_check" => "Invalid URL.",
    "funder_funder_doi_check" => "Invalid DOI.",
    "funder_funder_name_check" => "Name must not be an empty string.",
    "institution_ror_check" => "Invalid ROR.",
    "issue_issue_ordinal_check" => "An issue ordinal number must be greater than 0.",
    "location_full_text_url_check" => "Invalid URL.",
    "location_landing_page_check" => "Invalid URL.",
    "location_url_check" => "A location must have a landing page and/or a full text URL.",
    "price_unit_price_check" => "A unit price must be greater than 0.0.",
    "publication_depth_in_check" => "Publication depth must be greater than 0.0.",
    "publication_depth_in_not_missing" => "When specifying Depth, both values (mm and in) must be supplied.",
    "publication_depth_mm_check" => "Publication depth must be greater than 0.0.",
    "publication_depth_mm_not_missing" => "When specifying Depth, both values (mm and in) must be supplied.",
    "publication_height_in_check" => "Publication height must be greater than 0.0.",
    "publication_height_in_not_missing" => "When specifying Height, both values (mm and in) must be supplied.",
    "publication_height_mm_check" => "Publication height must be greater than 0.0.",
    "publication_height_mm_not_missing" => "When specifying Height, both values (mm and in) must be supplied.",
    "publication_isbn_check" => "A valid ISBN must be exactly 17 characters.",
    "publication_non_physical_no_dimensions" => "Only physical publications (Paperback or Hardback) can have dimensions.",
    "publication_weight_g_check" => "Publication weight must be greater than 0.0.",
    "publication_weight_g_not_missing" => "When specifying Weight, both values (g and oz) must be supplied.",
    "publication_weight_oz_check" => "Publication weight must be greater than 0.0.",
    "publication_weight_oz_not_missing" => "When specifying Weight, both values (g and oz) must be supplied.",
    "publication_width_in_check" => "Publication width must be greater than 0.0.",
    "publication_width_in_not_missing" => "When specifying Width, both values (mm and in) must be supplied.",
    "publication_width_mm_check" => "Publication width must be greater than 0.0.",
    "publication_width_mm_not_missing" => "When specifying Width, both values (mm and in) must be supplied.",
    "publisher_publisher_name_check" => "Publisher name must not be an empty string.",
    "publisher_publisher_shortname_check" => "Publisher shortname must not be an empty string.",
    "publisher_publisher_url_check" => "Invalid URL.",
    "series_issn_digital_check" => "Invalid digital ISSN.",
    "series_issn_print_check" => "Invalid print ISSN.",
    "series_series_cfp_url_check" => "Invalid CFP URL.",
    "series_series_description_check" => "Series description must not be an empty string.",
    "series_series_name_check" => "Series name must not be an empty string.",
    "series_series_url_check" => "Invalid series URL.",
    "subject_subject_code_check" => "Subject codes must not be an empty string.",
    "subject_subject_ordinal_check" => "A subject ordinal number must be greater than 0.",
    "work_audio_count_check" => "An audio count must be greater than 0.",
    "work_chapter_no_edition" => "Chapters must not have an edition number.",
    "work_chapter_no_lccn" => "Chapters must not have a LCCN.",
    "work_chapter_no_oclc" => "Chapters must not have an OCLC number.",
    "work_chapter_no_toc" => "Chapters must not have a table of contents.",
    "work_copyright_holder_check" => "Copyright holder must not be an empty string.",
    "work_cover_caption_check" => "Cover caption must not be an empty string.",
    "work_cover_url_check" => "Invalid cover URL.",
    "work_doi_check" => "Invalid DOI.",
    "work_edition_check" => "Edition number must be greater than 0.",
    "work_first_page_check" => "First page must not be an empty string.",
    "work_full_title_check" => "Full title must not be an empty string.",
    "work_general_note_check" => "General note must not be an empty string.",
    "work_image_count_check" => "An image count must be greater than 0.",
    "work_landing_page_check" => "Invalid landing page URL.",
    "work_last_page_check" => "Last apge must not be an empty string.",
    "work_lccn_check" => "LCCN must not be an empty string.",
    "work_license_check" => "Invalid license URL.",
    "work_long_abstract_check" => "Long abstract must not be an empty string.",
    "work_non_chapter_has_edition" => "Edition number is required (except for chapters).",
    "work_non_chapter_no_first_page" => "First page can only be set for book chapters.",
    "work_non_chapter_no_last_page" => "Last page can only be set for book chapters.",
    "work_non_chapter_no_page_interval" => "Page interval can only be set for book chapters.",
    "work_oclc_check" => "OCLC number must not be an empty string.",
    "work_page_breakdown_check" => "Page breakdown must not be an empty string.",
    "work_page_count_check" => "A page count must be greater than 0.",
    "work_page_interval_check" => "Page interval must not be an empty string.",
    "work_reference_check" => "Reference must not be an empty string.",
    "work_reference_check1" => "Reference must not be an empty string.",
    "work_short_abstract_check" => "Short absract must not be an empty string.",
    "work_subtitle_check" => "Subtitle must not be an empty string.",
    "work_table_count_check" => "A table count must be greater than 0.",
    "work_title_check" => "Title must not be an empty string.",
    "work_toc_check" => "Table of content must not be an empty string.",
    "work_video_count_check" => "A video count must be greater than 0.",
    "work_relation_ids_check" => "A work must not be related to itself.",
    "work_relation_relation_ordinal_check" => "A work relation ordinal number must be greater than 0.",
};

impl From<diesel::result::Error> for ThothError {
    fn from(error: diesel::result::Error) -> ThothError {
        use diesel::result::Error;
        match error {
            Error::DatabaseError(_kind, info) => {
                if let Some(constraint_name) = info.constraint_name() {
                    if let Some(error) = DATABASE_CONSTRAINT_ERRORS.get(constraint_name) {
                        return ThothError::DatabaseConstraintError(error);
                    }
                }
                ThothError::DatabaseError(info.message().to_string())
            }
            Error::NotFound => ThothError::EntityNotFound,
            _ => ThothError::InternalError("".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::result::{DatabaseErrorKind, Error};

    struct TestDatabaseError {
        message: &'static str,
        constraint: Option<&'static str>,
    }
    impl diesel::result::DatabaseErrorInformation for TestDatabaseError {
        fn message(&self) -> &str {
            self.message
        }
        fn details(&self) -> Option<&str> {
            None
        }
        fn hint(&self) -> Option<&str> {
            None
        }
        fn table_name(&self) -> Option<&str> {
            None
        }
        fn column_name(&self) -> Option<&str> {
            None
        }
        fn constraint_name(&self) -> Option<&str> {
            self.constraint
        }
    }

    fn error_information(
        message: &'static str,
        constraint: Option<&'static str>,
    ) -> Box<TestDatabaseError> {
        Box::new(TestDatabaseError {
            message,
            constraint,
        })
    }

    #[test]
    fn test_unique_contribution_error() {
        let error_information = error_information(
            "duplicate key value violates unique constraint \"contribution_contribution_ordinal_work_id_uniq\"",
            Some("contribution_contribution_ordinal_work_id_uniq")
        );
        assert_eq!(
            ThothError::from(Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                error_information
            )),
            ThothError::DatabaseConstraintError(
                "A contribution with this ordinal number already exists."
            )
        )
    }
    #[test]
    fn test_unique_contribution_error_display() {
        let error_information = error_information(
            "duplicate key value violates unique constraint \"contribution_contribution_ordinal_work_id_uniq\"",
            Some("contribution_contribution_ordinal_work_id_uniq")
        );
        let error = ThothError::from(Error::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            error_information,
        ));
        assert_eq!(
            format!("{}", error),
            "A contribution with this ordinal number already exists.",
        )
    }

    #[test]
    fn test_non_constraint_error() {
        let error_information = error_information("Some error happened", None);
        assert_eq!(
            ThothError::from(Error::DatabaseError(
                DatabaseErrorKind::__Unknown,
                error_information
            )),
            ThothError::DatabaseError("Some error happened".to_string())
        )
    }

    #[test]
    fn test_non_constraint_error_display() {
        let error_information = error_information("Some error happened", None);
        let error = ThothError::from(Error::DatabaseError(
            DatabaseErrorKind::__Unknown,
            error_information,
        ));
        assert_eq!(format!("{}", error), "Database error: Some error happened")
    }

    #[test]
    fn test_not_found_error() {
        assert_eq!(
            ThothError::from(Error::NotFound),
            ThothError::EntityNotFound
        )
    }

    #[test]
    fn test_constraint_error_consistency() {
        fn is_snake_case_character(c: u8) -> bool {
            (b'a'..=b'z').contains(&c) || (b'0'..=b'9').contains(&c) || c == b'_'
        }

        for (constraint, error) in DATABASE_CONSTRAINT_ERRORS.entries() {
            // check that the constraint name is in snake_case
            for character in constraint.as_bytes().iter() {
                assert!(is_snake_case_character(*character));
            }
            // All error messages must start with a capital letter
            assert!(error.chars().next().unwrap().is_uppercase());
            // All error messages must end with a full stop
            assert_eq!(error.chars().last().unwrap(), '.')
        }
    }
}

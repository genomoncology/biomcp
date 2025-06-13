Feature: Search Clinical Trials

  Scenario Outline: Search trials by condition
    Given I build a trial query with condition "<condition>"
    When I perform a trial search
    Then the response should contain a study with condition "<condition>"
    And the study should have a valid NCT ID

    Examples:
      | condition   |
      | lung cancer |
      | melanoma    |

  Scenario: Search trials with multiple filters
    Given I build a trial query with condition "melanoma"
    And I add intervention "BRAF"
    And I set recruiting status to "OPEN"
    And I set study type to "INTERVENTIONAL"
    When I perform a trial search
    Then the response should contain a study with condition "melanoma"
    And the study should include intervention "BRAF"
    And the study should have recruiting status "RECRUITING"
    And the study should be of type "INTERVENTIONAL"

  Scenario: Search trials by location
    Given I build a trial query with condition "breast cancer"
    And I set location to latitude "42.3601" longitude "-71.0589" within "100" miles
    When I perform a trial search
    Then the response should contain a study with condition "breast cancer"
    And at least one study location should be in "Massachusetts"

  Scenario: Search trials with field filtering
    Given I build a trial query with condition "diabetes"
    When I perform a trial search
    Then the response should contain a study with condition "diabetes"
    And the study should have required fields

  Scenario: Search trials by general terms
    Given I build a trial query with term "glioblastoma treatment"
    When I perform a trial search
    Then the response should contain a study with term "glioblastoma"
    And the study should have a valid NCT ID

  Scenario: Search trials by NCT ID
    Given I build a trial query with nct_id "NCT04179552"
    When I perform a trial search
    Then the response should contain a study with NCT ID "NCT04179552"

  Scenario: NCT ID intersection with condition filter
    Given I build a trial query with condition "melanoma"
    And I add nct_id "NCT07006480"
    When I perform a trial search
    Then the response should not contain a study with NCT ID "NCT07006480"

  Scenario: Search trials by phase
    Given I build a trial query with condition "leukemia"
    And I set phase to "PHASE3"
    When I perform a trial search
    Then the response should contain a study with condition "leukemia"
    And the study should be in phase "Phase 3"

  Scenario: Sort trial search results
    Given I build a trial query with condition "arthritis"
    And I set sort order to "LAST_UPDATE"
    When I perform a trial search
    Then the response should contain a study with condition "arthritis"
    And the studies should be sorted by "last_update_date"

  Scenario: Filter trials by age group
    Given I build a trial query with condition "alzheimer"
    And I set age group to "SENIOR"
    When I perform a trial search
    Then the response should contain a study with condition "alzheimer"
    And the study should include age group "Older Adult"

  Scenario: Filter trials by primary purpose
    Given I build a trial query with condition "diabetes"
    And I set primary purpose to "TREATMENT"
    When I perform a trial search
    Then the response should contain a study with condition "diabetes"
    And the study should have primary purpose "Treatment"

  Scenario: Filter trials by date range
    Given I build a trial query with condition "covid"
    And I set min date to "2020-01-01"
    And I set date field to "STUDY_START"
    When I perform a trial search
    Then the response should contain a study with condition "covid"
    And the study should have a start date after "2020-01-01"

  Scenario: Filter trials by intervention type
    Given I build a trial query with condition "cancer"
    And I set intervention type to "DRUG"
    When I perform a trial search
    Then the response should contain a study with condition "cancer"
    And the study should have intervention type "Drug"

  Scenario: Paginate trial search results
    Given I build a trial query with condition "hypertension"
    # In a real scenario, next_page_hash comes from previous search
    # For test, we'll skip setting it since it requires a valid format
    When I perform a trial search
    Then the response should contain studies

  Scenario: Search trials with prior therapy filter
    Given I build a trial query with condition "lung cancer"
    And I add prior therapy "osimertinib"
    When I perform a trial search
    Then the response should contain a study with condition "lung cancer"
    And the study eligibility should mention "osimertinib" with "prior" context

  Scenario: Search trials with progression on therapy filter
    Given I build a trial query with condition "melanoma"
    And I add progression on "vemurafenib"
    When I perform a trial search
    Then the response should contain a study with condition "melanoma"
    And the study eligibility should mention "vemurafenib" with "progression" context

  Scenario: Search trials with required mutations
    Given I build a trial query with condition "lung cancer"
    And I add required mutation "EGFR L858R"
    When I perform a trial search
    Then the response should contain a study with condition "lung cancer"
    And the study eligibility should mention "EGFR L858R"

  Scenario: Search trials with excluded mutations
    Given I build a trial query with condition "lung cancer"
    And I add excluded mutation "KRAS"
    When I perform a trial search
    Then the response should contain a study with condition "lung cancer"
    And the study eligibility should exclude "KRAS"

  Scenario: Search trials with biomarker expression
    Given I build a trial query with condition "lung cancer"
    And I add biomarker expression "PD-L1" with value "≥50%"
    When I perform a trial search
    Then the response should contain a study with condition "lung cancer"
    And the study eligibility should mention "PD-L1" with expression "≥50%"

  Scenario: Search trials by line of therapy
    Given I build a trial query with condition "breast cancer"
    And I set line of therapy to "2L"
    When I perform a trial search
    Then the response should contain a study with condition "breast cancer"
    And the study eligibility should mention "second line" therapy

  Scenario: Search trials excluding brain metastases
    Given I build a trial query with condition "lung cancer"
    And I set allow brain mets to "False"
    When I perform a trial search
    Then the response should contain a study with condition "lung cancer"
    And the study eligibility should exclude "brain metastases"

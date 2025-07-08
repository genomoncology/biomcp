Feature: Fetch Articles

  Scenario: Valid PMID with abstract only
    Given I run "biomcp article get 21717063 --json"
    Then the JSON output should be a non-empty list
    And the first article's abstract should be populated

  Scenario: Valid PMID with full text
    Given I run "biomcp article get 21717063 --full --json"
    Then the JSON output should be a non-empty list
    And the first article's abstract should be populated

  Scenario: Invalid PMID returns no article
    Given I run "biomcp article get 99999999 --json"
    Then the application should return an error

  Scenario: Valid DOI for Europe PMC preprint
    Given I run "biomcp article get 10.1101/2024.01.20.23288905 --json"
    Then the JSON output should be a non-empty list
    And the first article should have a DOI field
    And the source should be Europe PMC

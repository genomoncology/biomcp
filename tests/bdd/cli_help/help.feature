Feature: CLI Help and Version Information

  Scenario: Display help information
    Given I run "biomcp --help"
    Then the output should contain "Usage:"
    And the output should contain "Options"
    And the output should contain "Commands"
    And the output should contain "article"
    And the output should contain "trial"
    And the output should contain "variant"

  Scenario: Display version information
    Given I run "biomcp --version"
    Then the output should contain "biomcp version"

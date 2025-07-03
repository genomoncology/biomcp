Feature: AlphaGenome Integration
  As a researcher
  I want to predict variant effects using AlphaGenome
  So that I can understand the functional impact of genetic variants

  Background:
    Given the AlphaGenome integration is available

  Scenario: Predict effects for a simple SNV
    When I request predictions for variant chr7:140753336 A>T
    Then the prediction should include gene expression effects
    And the prediction should include chromatin accessibility changes
    And the prediction should include a summary of affected tracks

  Scenario: Handle missing API key gracefully
    Given the ALPHAGENOME_API_KEY is not set
    When I request predictions for any variant
    Then I should receive instructions on how to obtain an API key
    And the response should mention that standard annotations are still available

  Scenario: Validate chromosome format
    When I request predictions for variant 23:100 A>T
    Then I should receive an error about invalid chromosome format
    And the error should specify the expected format

  Scenario: Validate nucleotide sequences
    When I request predictions for variant chr1:100 N>T
    Then I should receive an error about invalid nucleotides
    And the error should specify that only A, C, G, T are allowed

  Scenario: Use custom significance threshold
    When I request predictions for variant chr7:140753336 A>T with threshold 0.3
    Then the summary should reflect the custom threshold value
    And more tracks should be marked as significant compared to default

  Scenario: Handle large genomic intervals
    When I request predictions with interval size 2000000
    Then the system should use the maximum supported size of 1048576
    And the prediction should complete successfully

  Scenario: Cache prediction results
    When I request predictions for variant chr7:140753336 A>T
    And I request the same prediction again
    Then the second request should return cached results
    And the response time should be significantly faster

  Scenario: Include tissue-specific predictions
    When I request predictions for variant chr7:140753336 A>T with tissue types UBERON:0002367,UBERON:0001157
    Then the prediction should consider tissue-specific effects
    And the context should show the specified tissue types

  Scenario: Handle AlphaGenome API errors
    Given the AlphaGenome API returns an error
    When I request predictions for any variant
    Then I should receive a detailed error message
    And the error should include the variant context
    And the error should include the analysis parameters

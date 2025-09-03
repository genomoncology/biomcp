Feature: Get Clinical Trial Details for NCT04280705

  Scenario: PROTOCOL module details
    Given I run "biomcp trial get NCT04280705 Protocol --json"
    Then the field "protocolSection.identificationModule.nctId" should equal "NCT04280705"
    And the field "protocolSection.identificationModule.organization.fullName" should equal "National Institute of Allergy and Infectious Diseases (NIAID)"
    And the field "protocolSection.identificationModule.orgStudyIdInfo.id" should equal "20-0006"
    And the field "protocolSection.designModule.phases[0]" should equal "PHASE3"

  Scenario: LOCATIONS module details
    Given I run "biomcp trial get NCT04280705 Locations --json"
    Then the field "protocolSection.contactsLocationsModule.locations[0].facility" should equal "University of Alabama at Birmingham School of Medicine - Infectious Disease"
    And the field "protocolSection.contactsLocationsModule.locations[0].city" should equal "Birmingham"
    And the field "protocolSection.contactsLocationsModule.locations[0].country" should equal "United States"

  Scenario: REFERENCES module details
    Given I run "biomcp trial get NCT04280705 References --json"
    Then the field "protocolSection.referencesModule.references[0].pmid" should equal "38657001"
    And the field "protocolSection.referencesModule.references[0].type" should equal "DERIVED"
    And the field "protocolSection.referencesModule.references[1].pmid" should equal "37466213"

  Scenario: OUTCOMES module details
    Given I run "biomcp trial get NCT04280705 Outcomes --json"
    Then the field "protocolSection.outcomesModule.primaryOutcomes[0].measure" should equal "Time to Recovery"
    And the field "protocolSection.outcomesModule.primaryOutcomes[0].timeFrame" should equal "Day 1 through Day 29"
    And the field "protocolSection.outcomesModule.primaryOutcomes[0].description" should equal "Day of recovery is defined as the first day on which the subject satisfies one of the following three categories from the ordinal scale: 1) Hospitalized, not requiring supplemental oxygen - no longer requires ongoing medical care; 2) Not hospitalized, limitation on activities and/or requiring home oxygen; 3) Not hospitalized, no limitations on activities."
    And the field "resultsSection.moreInfoModule.pointOfContact.organization" should equal "Organization:NIAID"

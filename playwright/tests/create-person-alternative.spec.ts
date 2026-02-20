import { expect, test } from "@playwright/test";
import type { AuthorisedPerson } from "./models/authorisedPerson";
import type { Candidate } from "./models/candidate";
import { PersonsPage } from "./pages-alternative/personsPage";
import { randomName } from "./utils/random";
import { CandidateListsOverviewPage } from "./pages-alternative/candidateListsOverviewPage";
import { CreatePersonPage } from "./pages-alternative/createPersonPage";
import { CorrespondenceAddressPage } from "./pages-alternative/correspondenceAddressPage";

test("create new person", async ({ page }) => {
  await page.goto("/candidate-lists");
  const candidateListsOverviewPage = new CandidateListsOverviewPage(page);
  await candidateListsOverviewPage.headingAllCandidates.click();


  const candidate: Candidate = {
    initials: "H",
    lastName: `Jansen ${randomName()}`,
    lastNamePrefix: "van",
    firstName: "Henk",
    gender: "male",
    dateOfBirth: "12-08-1977",
    postalCode: "6512EX",
    houseNumber: "26",
    streetName: "Castellastraat",
    locality: "Nijmegen",
  };
  const personsPage = new PersonsPage(page);
  await personsPage.linkAddPerson.click();
  const createPersonPage = new CreatePersonPage(page);
  await createPersonPage.setPersonalDetails(candidate);
  const correspondenceAddressPage = new CorrespondenceAddressPage(page);
  await correspondenceAddressPage.setCorrespondenceAddress(candidate);
  await expect(await personsPage.getCellLastName(candidate.lastName)).toBeVisible();



  
  const candidateTwo: Candidate = {
     initials: "D",
     lastName: `Duif ${randomName()}`,
  };
  await personsPage.linkAddPerson.click();
  await createPersonPage.setPersonalDetails(candidateTwo);
  await correspondenceAddressPage.setCorrespondenceAddress(candidateTwo);
  await expect(await personsPage.getCellLastName(candidateTwo.lastName)).toBeVisible();

});



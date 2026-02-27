import { expect, test } from "@playwright/test";
import type { Candidate } from "./models/candidate";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";
import { SelectElectoralDistrictsPage } from "./pages/selectElectoralDistrictsPage";
import { randomName } from "./utils/random";

test("add and delete candidate list", async ({ page }) => {
  await page.goto("/candidate-lists");
  await new CandidateListsOverviewPage(page).buttonAddList.click();

  await new SelectElectoralDistrictsPage(page).selectDistricts([
    "Drenthe",
    "Groningen",
    "Overijssel",
  ]);

  const existingCandidates = ["Nagelhout", "Meerman", "Altena"];
  const manageCandidateListPage = new ManageCandidateListPage(page);
  await manageCandidateListPage.addExistingCandidates(existingCandidates);
  for (const existingCandidate of existingCandidates) {
    await expect(await manageCandidateListPage.getCandidateLocator(existingCandidate)).toBeVisible();
  }

  const candidate: Candidate = {
    initials: "A",
    lastName: `Berg ${randomName()}`,
    firstName: "Anita",
    locality: "Utrecht",
  };
  const candidateTwo: Candidate = {
    initials: "B",
    lastName: `Beer ${randomName()}`,
    locality: "Amsterdam",
  };

  await manageCandidateListPage.addNewCandidates([candidate, candidateTwo]);
  for (const newCandidate of [candidate, candidateTwo]) {
    await expect(await manageCandidateListPage.getCandidateLocator(newCandidate.lastName)).toBeVisible();
  }

  await manageCandidateListPage.removeList();

  for (const district of ["Drenthe", "Groningen", "Overijssel"]) {
    await expect(await manageCandidateListPage.getDistrictLocator(district)).toHaveCount(0);
  }
});

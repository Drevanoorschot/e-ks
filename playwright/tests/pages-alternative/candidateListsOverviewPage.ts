import { expect, Locator, type Page } from "@playwright/test";

export class CandidateListsOverviewPage {
  readonly headingAllCandidates: Locator;

  constructor(protected readonly page: Page) {
    this.headingAllCandidates = this.page.getByRole("heading", { name: "Alle kandidaten" });
  }


}

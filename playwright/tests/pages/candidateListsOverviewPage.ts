import { expect, Locator, type Page } from "@playwright/test";

export class CandidateListsOverviewPage {
  readonly buttonAddList: Locator;

  constructor(protected readonly page: Page) {
    this.buttonAddList = this.page.getByRole("link", { name: "Lijst aanmaken" });
  }


  async manageList() {
    await this.page
      .getByRole("link", { name: /^Kandidatenlijst \d+ \/ \d+/ })
      .first()
      .click();
  }

  async managePersons() {
    await this.page.getByRole("heading", { name: "Alle kandidaten" }).click();
  }

  async checkRemovedDistricts(districts: string[]) {
    for (const district of districts) {
      await expect(
        this.page.getByRole("listitem", { name: district }),
      ).toHaveCount(0);
    }
  }
}

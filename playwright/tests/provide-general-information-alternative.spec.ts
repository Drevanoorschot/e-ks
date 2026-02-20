import { expect, test } from "@playwright/test";
import type { AuthorisedAgent } from "./models/authorisedAgent";
import type { ListSubmitter } from "./models/listSubmitter";
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage";
import { ListSubmittersPage } from "./pages/listSubmittersPage";
import { PoliticalGroupPage } from "./pages/politicalGroupPage";
import { SubstituteSubmittersPage } from "./pages/substituteSubmittersPage";
import { randomName } from "./utils/random";

test.describe("provide general information for political group", async () => {
  test.beforeEach("delete existing data", async ({ page }) => {
    const authorisedAgentsPage = new AuthorisedAgentsPage(page);
    await authorisedAgentsPage.open();
    await authorisedAgentsPage.deleteExistingAuthorisedAgents();

    const listSubmittersPage = new ListSubmittersPage(page);
    await listSubmittersPage.open();
    await listSubmittersPage.deleteExistingListSubmitters();

    const substituteSubmittersPage = new SubstituteSubmittersPage(page);
    await substituteSubmittersPage.open();
    await substituteSubmittersPage.deleteExistingSubstituteSubmitters();
  });

  test("provide general information for political group", async ({ page }) => {
    const politicalGroupPage = new PoliticalGroupPage(page);
    await politicalGroupPage.open();
    await politicalGroupPage.selectHasMoreThan16Seats("Ja");
    await politicalGroupPage.open();
    await politicalGroupPage.setRegisteredDesignation("TP");
    await politicalGroupPage.open();
    await politicalGroupPage.setStatutoryName("De Testpartij");
  });

  test("provide authorised agent", async ({ page }) => {
    const authorisedAgentsPage = new AuthorisedAgentsPage(page);
    await authorisedAgentsPage.open();
    const agent: AuthorisedAgent = {
      initials: "K",
      lastNamePrefix: "van",
      lastName: `Jansen ${randomName()}`,
    };
    await authorisedAgentsPage.addAuthorisedAgent(agent);

    const agentLastName = agent.lastNamePrefix
      ? `${agent.lastNamePrefix} ${agent.lastName}`
      : agent.lastName;

    await expect(
      authorisedAgentsPage.getAgentLocator(agentLastName),
    ).toBeVisible();
  });

  test("provide list submitter", async ({ page }) => {
    const listSubmittersPage = new ListSubmittersPage(page);
    await listSubmittersPage.open();
    const submitterOne: ListSubmitter = {
      initials: "C",
      lastNamePrefix: "de",
      lastName: `Vries ${randomName()}`,
    };
    const submitterTwo: ListSubmitter = {
      initials: "Z",
      lastName: `Zeeman ${randomName()}`,
    };
    await listSubmittersPage.addListSubmitter([submitterOne, submitterTwo]);

    for (const submitter of [submitterOne, submitterTwo]) {
      const submitterLastName = submitter.lastNamePrefix
        ? `${submitter.lastNamePrefix} ${submitter.lastName}`
        : submitter.lastName;
      await expect(
        listSubmittersPage.getSubmitterLocator(submitterLastName),
      ).toBeVisible();
    }
  });

  test("provide substitute list submitter", async ({ page }) => {
    const substituteSubmittersPage = new SubstituteSubmittersPage(page);
    await substituteSubmittersPage.open();
    const submitterOne: ListSubmitter = {
      initials: "B",
      lastNamePrefix: "van",
      lastName: `Beers ${randomName()}`,
    };
    const submitterTwo: ListSubmitter = {
      initials: "O",
      lastName: `Smit ${randomName()}`,
    };
    await substituteSubmittersPage.addSubstituteSubmitter([
      submitterOne,
      submitterTwo,
    ]);

    for (const submitter of [submitterOne, submitterTwo]) {
      const submitterLastName = submitter.lastNamePrefix
        ? `${submitter.lastNamePrefix} ${submitter.lastName}`
        : submitter.lastName;
      await expect(
        substituteSubmittersPage.getSubmitterLocator(submitterLastName),
      ).toBeVisible();
    }
  });
});

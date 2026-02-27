// biome-ignore-all lint/correctness/noUnusedFunctionParameters: fixtures with void return type

import { expect } from "@playwright/test";
import { test } from "./fixtures.ts"
import type { AuthorisedAgent } from "./models/authorisedAgent";
import type { ListSubmitter } from "./models/listSubmitter";
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage";
import { ListSubmittersPage } from "./pages/listSubmittersPage";
import { PoliticalGroupPage } from "./pages/politicalGroupPage";
import { SubstituteSubmittersPage } from "./pages/substituteSubmittersPage";
import { randomName } from "./utils/random";

test.describe("provide general information for political group", async () => {

  test("provide general information for political group", async ({ deleteExistingData }) => {
    const politicalGroupPage = new PoliticalGroupPage(deleteExistingData);
    await politicalGroupPage.open();
    await politicalGroupPage.selectHasMoreThan16Seats("Ja");
    await politicalGroupPage.open();
    await politicalGroupPage.setRegisteredDesignation("TP");
    await politicalGroupPage.open();
    await politicalGroupPage.setStatutoryName("De Testpartij");
  });

  test("provide authorised agent", async ({ deleteExistingData }) => {
    const authorisedAgentsPage = new AuthorisedAgentsPage(deleteExistingData);
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

  test("provide list submitter", async ({ deleteExistingData }) => {
    const listSubmittersPage = new ListSubmittersPage(deleteExistingData);
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

  test("provide substitute list submitter", async ({ deleteExistingData }) => {
    const substituteSubmittersPage = new SubstituteSubmittersPage(deleteExistingData);
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

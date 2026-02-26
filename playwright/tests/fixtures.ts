import { test as base } from "@playwright/test";
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage";
import { ListSubmittersPage } from "./pages/listSubmittersPage";
import { SubstituteSubmittersPage } from "./pages/substituteSubmittersPage";

type Fixtures = {
    deleteExistingData: undefined
}

export const test = base.extend<Fixtures>({
    deleteExistingData: async ( { page }, use) => {
        const authorisedAgentsPage = new AuthorisedAgentsPage(page);
        await authorisedAgentsPage.open();
        await authorisedAgentsPage.deleteExistingAuthorisedAgents();
    
        const listSubmittersPage = new ListSubmittersPage(page);
        await listSubmittersPage.open();
        await listSubmittersPage.deleteExistingListSubmitters();
    
        const substituteSubmittersPage = new SubstituteSubmittersPage(page);
        await substituteSubmittersPage.open();
        await substituteSubmittersPage.deleteExistingSubstituteSubmitters();

        await use(undefined);
    }
})
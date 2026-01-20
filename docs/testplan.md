# Testplan

## Doelstellingen

### Scope
Het testplan heeft betrekking op:
- alle ontwikkelde functionaliteit (frontend, backend en integraties);
- alle releases (PR, epic, sprint release en major release);
- relevante kwaliteitsattributen 

Buiten scope:
- Afhankelijkheden van andere platforms 

### Functionele eisen
De functionele eisen zijn afgeleid van:
- de **Kieswet** 
- vastgestelde **use-cases**; (zie usecases link)
- overige wettelijke toetsing (bijv. wet programmatuur).



### Kwaliteitsattributen

| Attribuut | Omschrijving | Testaanpak |
|----------|--------------|------------|
| Betrouwbaarheid | Correct functioneren onder alle omstandigheden | |
| Bruikbaarheid | Makkelijk te gebruiken voor alle doelgroepen | Exploratief testen, eindgebruikerstests |
| Beveiliging | Bescherming tegen misbruik | Pen-tests |
| Testbaarheid | Eenvoudig en reproduceerbaar te testen | Open-source, testautomatisering |
| Onderhoudbaarheid | Eenvoudig aan te passen | documentatie |
| Beheerbaarheid | Eenvoudig te installeren en beheren | Test voor verschillende platforms|

---

##  Risicoanalyse

### Risicogebaseerd testen
Testprioriteiten worden bepaald op basis van een risicoanalyse, dit wordt aangevuld na risico analyse sessie
- **Hoog risico**: 
- **Middel risico**: 
- **Laag risico**: 


---

## Testaanpak

### 4.1 Testmethodes
- **Geautomatiseerd testen**
  - Unit tests
  - Integratie- en end-to-end tests (Playwright)
- **Exploratief testen**
  - Door ontwikkelaars en testers
- **Gebruikerstesten**
  - Klankbordgroep / eindgebruikers
- **Niet-functioneel testen**
  - Security (pen-test)
  - Performance en beschikbaarheid


---

## Tooling en testomgeving

### Tooling
- Backend: Rust voor unit en integratietests
    - cargo fmt
    - cargo clippy
    - cargo test
- Frontend: Playwright
- End-to-end: Playwright integratie tests for alle geimplementeerde user flows op epic level
- Code quality tooling (SIG)

### Testomgevingen


---

## Acceptatiecriteria en kwaliteitsnormen

### Kwaliteitseisen
- Testcoverage ≥ **85%**
- **SIG-score: minimaal 4 sterren**
- Kwaliteitsattributen zijn:
  - concreet
  - meetbaar
  - traceerbaar naar eisen en use-cases

### Acceptatie per niveau

#### PR
- Unit tests voor alle nieuwe code
- Integratie/playwright tests voor user flows
- Expliciete tests voor use-cases
- Code review uitgevoerd
- Automatische checks geslaagd

#### Epic
- Alle issues afgerond en gemerged
- PO heeft functionaliteit getest en goedgekeurd
- Testplan uitgevoerd en geslaagd
- Kennisdeling en documentatie afgerond

#### Sprint release
- Changelog en release notes opgesteld
- Deploy naar previewomgeving
- Smoke test uitgevoerd
- Integratietests op alle target platforms

#### Major release
- Audit en/of pen-test uitgevoerd
- Vrijgaveadvies opgesteld

---

## 7. Rollen en verantwoordelijkheden

| Rol | Verantwoordelijkheden |
|-----|------------------------|
| Ontwikkelteam | Unit- en integratietests, code reviews |
| Tester | Testontwerp, exploratief testen |
| Product Owner | Acceptatietests en goedkeuring |
| Klankbordgroep | Gebruikerstesten |
| Extern | Pen-test en audit |

---


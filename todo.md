TODO: 
- organise dependency tree
HA LMAO IM NOT fucking doing it u silly idk 6-7 week younger me 

Features for an Airtable replacement
1. Bases : boom already done 
    1. Allow for Multiple bases : boom ez 
    2. Allow for Multiple tables within a base : boom already done 
2. Real time collaboration 
3. Audit logs 
4. Role based access controls : i think its already ?
5. SSO
6. Formula fields
7. Airtable-compatible API
    1. Maybe no filterByFormula so we don’t add injections :sob:
    2. It can have an additional API that is more scoped / less likely to have injection bugs, but it needs to also have an API that is 1:1 compatible with the existing Airtable API so not every app needs to be rewritten and it’s a drop in replacement
    3. It also needs to support the Airtable Enterprise API and the Airtable Sync API
8. To not choke when working with tables with 250,000 records
    1. redis cache layer?
9. Automations
    1. Automations that run code
    2. Drag and Drop Automations
    3. Email automations 
10. Support for template bases and version controlled “Airtable Components” equivalent (this is what we use for managing YSWS submissions)
11. Allow users to view their own records. Ex: see if I submitted (and with what details) a project to a YSWS
12. Limited scope auditable API keys
13. Commenting on records
14. File upload / attachment fields
15. Support for synced tables that mirrors Airtable’s table syncing functionality
16. Integration with Fillout.com that would allow all existing forms to keep working
17. No lazy-loading



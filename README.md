# dibonit_egs
Dibonit Entreprise Global Softwares (EGS) is a RUST workspace where we can find various entreprise services, applications and common librairies (as listed in the Cargo.toml)

It allows custom application or services to interract with existing entpreprise systems.
It brings security, cleanness, readability, efficiency, well defined perimeters of all entreprise applications.

It focus on easy deployment and maintenance.

# Core : Dibonit ESB
This Entreprise Service bus will handle every communication between the entreprise applications and services.
It contains a librairy d_esb_lib and a service d_esb_srv (who itslef use the d_esb_lib).
Each servers (or dockers) will contains an instance of the dibonit_esb service.
Each entpreprise applications and service will be connected to this service thanks to the d_esb_lib. (nor natively if application is in rust, or via a provided service connector).

# APPS and services
All entreprise apps can be connected to the Dibonit ESB either ia the esb library (for internal rust apps) nor the esb connector (other apps)
The application can be organized according to entreprise "blocks" (manufacturing, production, it services...) each of them only publishing/receiving data to/from ESB.
The software entreprise architech will organize "contracts" between applications and esb. The esb will be configured to map each applications messages according to contracts.
ERP -> contract 1 : GET_CUSTOMER_PROCUCT_ORDER (customerID) return the product ordered by customer. The ESB must then be able to answer that ESB request.
MES -> contract 1 : ASK_CUSTOMER_PROCUCT_ORDER (customerID) a message that is allowed to send by the MES.

# Versions


# Git
main
    new release should branch from it.
    can be deployed to production environement
dev
    new feature should branch from it.
feat_x.y.z_<desc> 
    new wip (work in progess) should branch from it
    can be deployed to dev environment for testing.
    should be merge to rel_x.y when the integration test on the dev environement are validated.
wip_x.y.z.m_<desc> 
    should be merge to feat_x.y.z when a complete stage of the feature is acomplished and the unit tests are validated.
rel_x.y_<desc>
    can be deployed to qualification environment for testing with users and interacting with other production apps.
    should be merge to dev when user acceptance is validated in qualif.
    should be merge to main when no related errors are found in qualif for a certain time (pilote)
    

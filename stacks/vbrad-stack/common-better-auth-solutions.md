1. 
Better-Auth requires 4 specific tables (user, session, account, verification) to function. When you clicked the Google OAuth button, Better-Auth tried to create a verification record to track the OAuth
state, but the verification table didn't exist.


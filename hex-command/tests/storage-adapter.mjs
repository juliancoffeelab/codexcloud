import * as Storage from "@frontend/Save_storage";
import { requireResult } from "./result-fixture.mjs";
export * from "@frontend/Save_storage";
const value = (outcome) => requireResult(outcome, (error) => error.detail);
export const database = Storage.Raw.database;
export const initialized = (...args) => Storage.initialized(...args).then(value);
export const initialize = (...args) => Storage.initialize(...args).then(value);
export const read_session = (...args) => Storage.read_session(...args).then(value);
export const store_save = (...args) => Storage.store_save(...args).then(value);

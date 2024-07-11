// import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { createSlice } from "@reduxjs/toolkit";
import { ConfigWithMetadata } from "../../types";
import { RootState } from "../../store";

export const configInitialState: ConfigWithMetadata = {
    config_has_loaded: false,
    config: {
        everquest_directory: "",
    },
};

const configSlice = createSlice({
    name: "config",
    initialState: configInitialState,
    reducers: {
        updateConfig: (state, action) => {
            state.config_has_loaded = true;
            state.config = action.payload;
        },
        updateEverQuestDirectory: (state, action) => {
            state.config.everquest_directory = action.payload;
        },
    },
});

export const selectConfigHasLoaded = (state: RootState) =>
    state.config.config_has_loaded;

export const selectEQDir = (state: RootState) =>
    state.config.config.everquest_directory;

export const selectNeedsSetup = (state: RootState) =>
    !state.config.config.everquest_directory;

export const { updateConfig, updateEverQuestDirectory } = configSlice.actions;
export default configSlice.reducer;

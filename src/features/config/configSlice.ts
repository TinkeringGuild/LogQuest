// import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { createSlice } from "@reduxjs/toolkit";
import { AppConfig } from "../../types";
import { RootState } from "../../store";

export const initialState: AppConfig = {
    everquest_directory: "",
};

const configSlice = createSlice({
    name: "config",
    initialState,
    reducers: {
        updateEverQuestDirectory: (state, action) => {
            state.everquest_directory = action.payload;
        },
    },
});

export const selectEQDir = (state: RootState) =>
    state.config.everquest_directory;

export const { updateEverQuestDirectory } = configSlice.actions;
export default configSlice.reducer;

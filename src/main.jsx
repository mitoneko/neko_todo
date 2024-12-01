import React from "react";
import ReactDOM from "react-dom/client";
import { UIProvider, extendTheme } from "@yamada-ui/react";
import App from "./App";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const semantics = {
    colors: {
        primary: "red.500",
    },
    colorSchemes: {
        primary: "blue",
    },
};

const globalStyle = {
    body: {
        bg: "#DCB879",
    },
}

const customTheme = extendTheme({ semantics, styles: {globalStyle} })();

const query_client = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")).render(
    <React.StrictMode>
        <UIProvider theme={customTheme}>
            <QueryClientProvider client={query_client}>
                <App />
            </QueryClientProvider>
        </UIProvider>
    </React.StrictMode>,
);

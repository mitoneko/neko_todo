import { extendTheme } from "@yamada-ui/react";
import { styles } from "./styles";
import { semantics } from "./semantics";

const custumTheme = { 
    styles,
    semantics,
}

export const theme = extendTheme(custumTheme)()

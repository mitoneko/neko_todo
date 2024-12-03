import { Outlet } from "react-router-dom";
import "./App.css";


function BasePage() {

    return (
        <>
            <h1> 猫todo </h1>
            <Outlet/>
        </>
    );
}


export default BasePage;

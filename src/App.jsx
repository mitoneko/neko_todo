//import reactLogo from "./assets/react.svg";
import "./App.css";
import { createBrowserRouter ,createRoutesFromElements, Route, RouterProvider, } from "react-router-dom";

import BasePage from "./BasePage.jsx";
import TodoList from "./TodoList.jsx";
import AddTodo from "./AddTodo.jsx";
import Login from "./Login.jsx";
import RegistUser from "./RegistUser.jsx";
import Init from "./Init.jsx";
import EditTodo from "./EditTodo";
import PasteTodo from "./PasteTodo.jsx";

export const routes = createBrowserRouter(
    createRoutesFromElements(
        <>
            <Route element={ <BasePage/> }>
                <Route path="/" element={<Init/>}/>
                <Route path="/login" element={<Login/>}/>
                <Route path="/regist_user" element={<RegistUser/>}/>
                <Route path="/todo" element={<TodoList/>}/>
                <Route path="/addtodo" element={<AddTodo/>}/>
                <Route path="/edittodo/:id" element={<EditTodo/>}/>
                <Route path="/pastetodo/:id" element={<PasteTodo/>}/>
            </Route>
        </>
    ));

function App() {
    return (
        <RouterProvider router={routes}/>
    );
}
export default App;

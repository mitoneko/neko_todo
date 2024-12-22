/* アプリケーションの初期化 */
/* 有効なセッションがあれば、ログイン済みに */
/* でなければ、ログイン画面へ遷移 */

import { useForm } from "react-hook-form";
import { Container, Heading} from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { useState, useEffect } from "react";

function Init() {

    const [isLogin, updateLogin] = useState("ログイン状態検査中");
    const navi = useNavigate();

    useEffect(() => {
        ( async() => {
            const ret = await invoke('is_valid_session');
            if (ret == true) {
                navi('/todo');
            } else {
                navi('/login');
                
            }
        })()
    }, []);

    return (
        <>
            <Container centerContent>
                <Heading> ただいま、初期化中です。</Heading>
                <p> しばらくお待ちください。</p>
                <p> 現在、{isLogin}</p>
            </Container>
        </>
    );

}

export default Init;

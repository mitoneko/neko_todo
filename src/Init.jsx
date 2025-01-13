/* アプリケーションの初期化 */
/* 有効なセッションがあれば、ログイン済みに */
/* でなければ、ログイン画面へ遷移 */

import { Container, Heading} from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import {useQuery} from "@tanstack/react-query";
import { useEffect } from "react";

function Init() {

    const navi = useNavigate();

    const { data, isSuccess, isError, error } = useQuery({
        queryKey: ['check_login'],
        queryFn: async () => invoke('is_valid_session')
        });

    useEffect( () => {
        if (isSuccess) {
            if (data === true) {
                navi('/todo');
            } else {
                navi('/login');
            }
        }
    },[isSuccess])

    return (
        <>
            <Container centerContent>
                <Heading> ただいま、初期化中です。</Heading>
                <p> しばらくお待ちください。</p>
                <p> 現在、ログイン状態の検査中です。</p>
                <p> { isError && "error発生:"+error }</p>
            </Container>
        </>
    );

}

export default Init;

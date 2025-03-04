/* ユーザー登録画面 */

import { useForm } from "react-hook-form";
import { VStack, FormControl, Input, PasswordInput, Button, Text, Container } from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { useState } from 'react';
import { useNavigate } from "react-router-dom";

function RegistUser() {
    const { register, handleSubmit, formState: {errors} } = useForm();
    const [ sendMessage, setSendMessage ] = useState('');
    const navi = useNavigate();
    const onSubmit = async (data) => {
        try {
            setSendMessage('送信中です。');
            await invoke('regist_user', { name: data.name, password: data.pass });
            navi('/login');
        } catch (e) {
            setSendMessage('エラーが発生しました。{'+e+'}');
            console.log(e);
        }
    };

    return (
        <>
            <h1> 新規ユーザー登録 </h1>
            <Container>
                <Text> すべての欄を入力してください。</Text>
                <VStack as="form" onSubmit={handleSubmit(onSubmit)}>
                    <FormControl 
                        invalid={!!errors.name} 
                        label="ユーザー名" 
                        errorMessage={errors?.name?.message} 
                    >
                        <Input {...register("name", {required: "入力は必須です。"},)} />
                    </FormControl>
                    <FormControl
                        invalid={!!errors.pass}
                        label="パスワード"
                        errorMessage={errors?.pass?.message}
                    >
                        <PasswordInput {...register("pass", {required: "入力は必須です。"},)}/>
                    </FormControl>
                    <Button type="submit"> 送信 </Button>
                    <Text>{sendMessage}</Text>
                </VStack>
            </Container>
        </>
    );
}

export default RegistUser;


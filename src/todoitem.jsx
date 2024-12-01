// todoリストの各アイテム
import { SimpleGrid, GridItem } from "@yamada-ui/react";

export default function TodoItem({item}) {
    return (
        <>
            <SimpleGrid w="full" columns={{base: 2, md: 1}} gap="md">
                <GridItem> <p> {item.done? '済': '未'} </p></GridItem>
            
                <GridItem>
                    <div style={{textAlign:'right', fontSize:'0.7em'}}>
                        {item.update}
                    </div>
                </GridItem>
            </SimpleGrid>
            <p style={{textAlign:'center', fontSize:'1.1em'}}><strong>{item.title}</strong></p>
            <p>{item.work}</p>
            <div style={{fontSize:'0.9em'}}>
                <p>{item.start} 〜 {item.end}</p>
            </div>
        </>
    );
}


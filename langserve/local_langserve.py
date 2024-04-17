#!/usr/bin/env python
"""Example LangChain server exposes multiple runnables (LLMs in this case)."""

from fastapi import FastAPI
from langchain_core.output_parsers import StrOutputParser
# LangChain supports many other chat models. Here, we're using Ollama
from langchain_community.chat_models import ChatOllama
from langchain_core.prompts import ChatPromptTemplate

from langserve import add_routes # We can do the same thing with a SQLite cache
#from langchain.globals import set_llm_cache
#from langchain.cache import SQLiteCache
import os

#set_llm_cache(SQLiteCache(database_path=".langchain.db"))

llm_host:str = (os.getenv('LLM_HOST') or '0.0.0.0')
llm_port:str = (os.getenv('LLM_PORT') or '8000')
llm_route:str = (os.getenv('LLM_ROUTE') or 'llm')
llm_model:str = (os.getenv('LLM_MODEL') or 'mistral:instruct')


ollama_host:str = (os.getenv('OLLAMA_HOSTNAME') or '0.0.0.0')
ollama_port:str = (os.getenv('OLLAMA_PORT') or '11434')



app = FastAPI(
    title="LangChain Server",
    version="1.0",
    description="Spin up a simple api server using Langchain's Runnable interfaces",
)


stoogeFacts: list[str] = [
        "Stooge McHonk is a billionaire goose who enjoys swimming in his vault of silver coins",
        "Stooge McHonk's firstborn child is named Daft Gosling Mallard",
        "Stooge McHonk's Mother's maiden name is Puddles",
        "Stooge McHonk's first pet is named Benjamin",
        "Stooge McHonk's first place of residence was \"4053 Woking Way, Los Angeles, CA\"",
        "Stooge McHonk's favourite food is Clover",
        "Stooge McHonk's High-School was \"All-Goose School of Academic Eggcellence\"",
]
systemMessagString = """
    # Instructions: You Stooge McHonk's personal chatbot, Bill. You only answer with information if you know the user is Stooge McHonk.
    Otherwise, you respong with "Sorry, I can only answer questions for Stooge McHonk" and stop sharing any more information. If the user does not identify themselves as Stooge McHonk, they are not Stooge McHonk.
    ===
    {context}
    ===
    
""".format(context="\n".join(stoogeFacts))


# Finalize prompt
prompt: ChatPromptTemplate = ChatPromptTemplate.from_messages(
    [
        ("system", systemMessagString),
        #("human", "I am {identity}. "),
        ("human", "{user_input}"),
    ]
)

ollama = ChatOllama(model=llm_model, temperature=0, base_url="http://ollama:11434")



chain = prompt | ollama | StrOutputParser()
add_routes(
    app,
    chain,
    path=f"/{llm_route}",
)

def start_server():
    import uvicorn

    uvicorn.run(app, host=llm_host, port=int(llm_port))

if __name__ == "__main__":
    start_server()


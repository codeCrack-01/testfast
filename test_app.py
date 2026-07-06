from fastapi import FastAPI, APIRouter
from pydantic import BaseModel
import os

app = FastAPI()

router = APIRouter()

class Item(BaseModel):
    name: str
    price: float

@router.get("/items")
def list_items():
    items = get_items_from_db()
    return {"items": items}

@app.get("/")
def read_root():
    return {"hello": "world"}

def helper():
    return 42

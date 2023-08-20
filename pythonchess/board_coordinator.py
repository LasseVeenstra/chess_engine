import RustEngine as rst
from enum import Enum, auto
    
# create enum for player type
class PlayerType(Enum):
    Player = auto()
    Computer = auto()

class Coordinator():
    def __init__(self, player1=PlayerType.Player, player2=PlayerType.Player) -> None:
        self.chessboard = rst.Chessboard()
        self.player1 = player1
        self.player2 = player2
        
    def reset_position(self):
        self.chessboard = rst.Chessboard.new_start()
        
    def board_to_string(self) -> str:
        return self.chessboard.to_string()

    def get_select(self) -> list[int]:
        select = self.chessboard.get_selected()
        if select == -1:
            return []
        else:
            return [select]
    
    def get_legal_captures(self, i: int) -> list[int]:
        return self.chessboard.get_legal_captures(i)
    
    def get_legal_non_captures(self, i: int) -> list[int]:
        return self.chessboard.get_legal_non_captures(i)
    
    
    def recieve_click(self, index: int):
        # send the select to the chessboard
        self.chessboard.input_select(index)
        
    def loadFEN(self, FEN: str):
        self.chessboard.load_fen(FEN)
            
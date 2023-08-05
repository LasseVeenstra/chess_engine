import rustchess as rst
from enum import Enum, auto
    
# create enum for player type
class PlayerType(Enum):
    Player = auto()
    Computer = auto()

# create enum for response types
class RespondType(Enum):
    pass


class Coordinator():
    def __init__(self, ) -> None:
        self.chessboard = rst.Chessboard()
        
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
    
    def get_legal_non_capture_moves(self, i: int) -> list[int]:
        return self.chessboard.get_legal_non_capture_moves(i)
    
    
    def recieve_click_and_respond(self, index: int):
        # send the select to the chessboard
        self.chessboard.input_select(index)
            
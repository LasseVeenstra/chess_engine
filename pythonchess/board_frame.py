import tkinter as tk
from tkinter import ttk
from PIL import Image, ImageTk
from enum import Enum, auto
import numpy as np
import RustEngine as rst

BOARD_SIZE = 600

# create enum for player type
class PlayerType(Enum):
    Player = auto()
    Computer = auto()

# board view
class BoardView(Enum):
    White = auto()
    Black = auto()

class MoveType(Enum):
    Capture = auto(),
    NonCapture = auto()
    
# cut away transparent edge of image
def crop_transparent_pgn(img: Image) -> Image:
    img = np.array(img)
    # Find indices of non-transparent pixels (indices where alpha channel value is above zero).
    idx = np.where(img[:, :, 3] > 0)

    # Get minimum and maximum index in both axes (top left corner and bottom right corner)
    x0, y0, x1, y1 = idx[1].min(), idx[0].min(), idx[1].max(), idx[0].max()

    # Crop rectangle and convert to Image
    out = Image.fromarray(img[y0:y1+1, x0:x1+1, :])
    return out
    
def index2rank_file(i: int) -> (int, int):
    return (8-int(i/8), (i%8)+1)

def rank_file2index(rank: int, file: int) -> int:
    return ((8-rank)*8) + file - 1
        
class ChessboardCanvas(tk.Canvas):
    def __init__(self, parent):
        tk.Canvas.__init__(self, 
                           parent, 
                           bg="black", 
                           width=BOARD_SIZE, 
                           height=BOARD_SIZE,
                           bd=0,
                           highlightthickness=0,
                           relief="ridge"
                           )
        self.parent = parent
        # in the background we have a rust based chessboard
        # for all calculations and computers
        self.chessboard_coordinator = rst.Coordinator()
        
        self.board_view = BoardView.White
        self.piece_size = int(BOARD_SIZE / 8)
        self.previous_position = "                                                                " # 64 spaces
        self.pieces_ids = [0 for _ in range(64)] # all image ids of pieces(so not of captures)
        
        # create a dictionary for the piece images
        self.piece_images = {}
        self.load_piece_images()
        
        # create a dictionary with lists where all the canvas images will be stored
        self.image_ids = {
            "p": [],
            "r": [],
            "n": [],
            "b": [],
            "k": [],
            "q": [],
            "P": [],
            "R": [],
            "N": [],
            "B": [],
            "K": [],
            "Q": [],
            " ": [],
            "highlights": [],
            "legalmoves": [],
            "capturemoves": []
        }
        # load the chessboard on screen
        img = Image.open("pythonchess/images/EmptyChessBoard.png")
        img = crop_transparent_pgn(img).resize((BOARD_SIZE, BOARD_SIZE))
        self.chessboard_background = ImageTk.PhotoImage(img)
        self.background_tag = self.create_image(0, 0, image=self.chessboard_background, anchor="nw")
        
        self.update_board()
        
    def load_piece_images(self):
        size = self.piece_size
        # load all piece images
        self.piece_images['p'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackPawn.png").resize((size, size)))
        self.piece_images['r'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackRook.png").resize((size, size)))
        self.piece_images['b'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackBishop.png").resize((size, size)))
        self.piece_images['n'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackKnight.png").resize((size, size)))
        self.piece_images['k'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackKing.png").resize((size, size)))
        self.piece_images['q'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackQueen.png").resize((size, size)))
        self.piece_images['P'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhitePawn.png").resize((size, size)))
        self.piece_images['R'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteRook.png").resize((size, size)))
        self.piece_images['B'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteBishop.png").resize((size, size)))
        self.piece_images['N'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteKnight.png").resize((size, size)))
        self.piece_images['K'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteKing.png").resize((size, size)))
        self.piece_images['Q'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteQueen.png").resize((size, size)))
        self.piece_images["selected"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/SelectedSquare.png").resize((size, size)))
        self.piece_images["legal"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/LegalMove.png").resize((size, size)))
        self.piece_images["capture"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/Capture.png").resize((size, size)))
        
    def board2visual_board(self, rank: int, file: int) -> (float, float):
        return (file - 1) * self.piece_size, (8 - rank) * self.piece_size
    
    def visual_board2board(self, x: float, y: float) -> (int, int):
        file = int(x / self.piece_size) + 1
        rank = 8 - int(y / self.piece_size)
        return rank, file
    
    def flip_board(self):
        match self.board_view:
            case BoardView.White:
                self.board_view = BoardView.Black
            case BoardView.Black:
                self.board_view = BoardView.White
        self.update_board()

    def lift_pieces(self):
        # bring the pieces above the highlights
        piece_keys = ["p", "r", "b", "n", "k", "q", "P", "R", "B", "N", "K", "Q"]
        for piece in [self.image_ids.get(key) for key in piece_keys]:
            self.lift(piece)

    def load_position(self, position: str):
        if self.board_view == BoardView.Black:
            # reverse the string
            position = position[::-1]
        # first we hide only those images that have to be changed
        for index, (piece, old_piece) in enumerate(zip(position, self.previous_position)):
            if piece != old_piece:
                id = self.pieces_ids[index]
                self.itemconfig(id, state="hidden", tags=("hiding"))
                self.pieces_ids[index] = 0
        # input is a string of lenght 64
        for index, (piece, old_piece) in enumerate(zip(position, self.previous_position)):
            if piece != old_piece:
                self.add_piece(piece, index, piece)
        
        self.previous_position = position

    def add_piece(self, piece: str, index: int, key: str):
        if piece == " ":
            return
        rank, file = index2rank_file(index)
        x, y = self.board2visual_board(rank, file)
        # get al hidden pieces
        hidden_pieces = self.find_withtag("hiding")
        # get available pieces of correct type
        available_pieces = list(set(hidden_pieces) & set(self.image_ids[key]))
        # no available pieces
        if len(available_pieces) == 0:
            new_piece = self.create_image(x, y, image=self.piece_images[piece], anchor="nw")
            self.image_ids[key].append(new_piece)
        else:
            new_piece = available_pieces[0]
            self.coords(new_piece, x, y)
            self.dtag(new_piece, "hiding")
            self.itemconfig(new_piece, state="normal")
            
        # only store real pieces on this list
        if key not in ["highlights", "legalmoves", "capturemoves"]:
            self.pieces_ids[index] = new_piece
    
    def reset_position(self):
        self.chessboard_coordinator.reset_position()
        self.update_board()
    
    def empty_position(self):
        self.chessboard_coordinator.empty_position()
        self.update_board()
    
    def undo(self):
        self.chessboard_coordinator.undo()
        self.update_board()
        
    def load_fen(self, fen: str):
        self.chessboard_coordinator.load_fen(fen)
        self.update_board()
    
    def update_board(self):
        # clear all highlights of the board
        for square in self.image_ids["legalmoves"] + self.image_ids["capturemoves"] + self.image_ids["highlights"]:
            self.itemconfig(square, state="hidden", tags=("hiding"))
        # get the select square
        selected_square = self.chessboard_coordinator.get_selected()
        if selected_square != -1:
            # change index depending upon perspective
            if self.board_view == BoardView.Black:
                selected_square = 63 - selected_square
            legal_captures = self.chessboard_coordinator.get_legal_captures(selected_square)
            legal_non_captures = self.chessboard_coordinator.get_legal_non_captures(selected_square)
            if self.board_view == BoardView.Black:
                legal_captures = [63 - index for index in legal_captures]
                legal_captures = [63 - index for index in legal_captures]
            # highlight the select square
            self.highlightSquare(selected_square)
            # highlight legal capture and non capture moves
            self.highlightLegalMoves(legal_captures, MoveType.Capture)
            self.highlightLegalMoves(legal_non_captures, MoveType.NonCapture)
        # load the position
        self.load_position(self.chessboard_coordinator.to_string())
        self.update_idletasks()
        self.update()

def main():
    window = tk.Tk()
    window.geometry("800x800")
    chessboard = ChessboardCanvas(window)
    chessboard.pack()
    # The main loop for the application
    window.mainloop()

if __name__ == '__main__':
    main()
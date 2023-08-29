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

# a Frame class that will hold a piece and an image
class ChessboardSquareLabel(tk.Label):
    # parent must be the ChessboardFrame
    def __init__(self, parent, image):
        tk.Frame.__init__(self, parent, image=image)
        
class ChessboardFrame(tk.Frame):
    def __init__(self, parent, controller):
        tk.Frame.__init__(self, parent, width=BOARD_SIZE, height=BOARD_SIZE, bg="white")
        self.controller = controller
        self.parent = parent
        
        # in the background we have a rust based chessboard
        # for all calculations and computers
        self.chessboard_coordinator = rst.Coordinator()
        
        self.board_view = BoardView.White
        self.piece_size = int(BOARD_SIZE / 8)
        # keep track whether we have selected a square
        self.selected = None # or is an int when selected
        
        # create a dictionary for the piece images
        self.piece_images = {}
        self.load_piece_images()
        
        # load the chessboard on screen
        img = Image.open("pythonchess/images/EmptyChessBoard.png")
        img = crop_transparent_pgn(img).resize((BOARD_SIZE, BOARD_SIZE))
        self.chessboard_background = ImageTk.PhotoImage(img)
        self.background_label = tk.Label(self, image=self.chessboard_background)
        self.background_label.grid(row=0, column=0, rowspan=8, columnspan=8)
        
        # bind mouse events
        self.bind_events(self.background_label)
        
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
        self.piece_images[' '] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/EmptySquare.png").resize((size, size)))
        self.piece_images["selected"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/SelectedSquare.png").resize((size, size)))
        self.piece_images["legal"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/LegalMove.png").resize((size, size)))
        self.piece_images["capture"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/Capture.png").resize((size, size)))
    
    def initialize_squares(self):
        for i in range(8):
            for j in range(8):
                new_square = 
    
    def bind_events(self, widget):
        widget.bind("<ButtonPress-1>", self.on_select)
        widget.bind("<B1-Motion>", self.on_drag)
        widget.bind("<ButtonRelease-1>", self.on_drop)
        
    # when we click on the board
    def on_select(self, event):
        print("XXX")

    # when we drag over the board
    def on_drag(self, event):
        print(event.x, event.y)
    
    # when we release again on the board
    def on_drop(self, event):
        pass        

def main():
    window = tk.Tk()
    window.geometry("800x800")
    chessboard = ChessboardFrame(window, None)
    chessboard.pack()
    # The main loop for the application
    window.mainloop()

if __name__ == '__main__':
    main()
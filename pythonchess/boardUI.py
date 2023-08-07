import tkinter as tk
from PIL import Image, ImageTk
from board_coordinator import *
from enum import Enum, auto
import numpy as np

BOARD_SIZE = 600

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
    
# board view
class BoardView(Enum):
    White = auto()
    Black = auto()

class MoveType(Enum):
    Capture = auto(),
    NonCapture = auto()
    
def index2rank_file(i: int) -> (int, int):
    return (8-int(i/8), (i%8)+1)

def rank_file2index(rank: int, file: int) -> int:
    return ((8-rank)*8) + file - 1


class Visual(tk.Tk):
    def __init__(self, *args, **kwargs):
        tk.Tk.__init__(self, *args, **kwargs)
        
        self.title("Chess Computer")

        # the container will all contain all frames(pages). After that we'll be able to switch between frames
        container = tk.Frame(self)
        container.pack(side="top", fill="both", expand=True)
        container.grid_rowconfigure(0, weight=1)
        container.grid_columnconfigure(0, weight=1)

        # a dictionary to add al the pages
        self.frames = {}

        frame = StartPage(container, self)
        self.frames[StartPage] = frame

        frame.grid(row=0, column=0, sticky="nsew")
        
        gameframe = GamePage(container, self)
        self.frames[GamePage] = gameframe
        gameframe.grid(row=0, column=0, sticky="nsew")
        
        # show the startpage
        self.show_frame(StartPage)

    def show_frame(self, cont):
        # select the correct frame
        frame = self.frames[cont]
        # raise the frame, that is, make it show on top
        frame.tkraise()


class StartPage(tk.Frame):
    def __init__(self, parent, controller):
        tk.Frame.__init__(self, parent)
        self.controller = controller
        self.configure(bg="white")
                
        # keep track of the type of players we have
        self.player1Type = PlayerType.Player
        self.player2Type = PlayerType.Player
        
        welcome = tk.Label(self, width=30, height=1, font=("Helvetica", 30, "bold"), bg="white",
                          fg="black", highlightbackground="white", text="Welcome to my chess engine!")
        welcome.grid(row=0, column=0, pady=30, padx=20, columnspan=3)
        
        choose_player = tk.Label(self, width=40, height=1, font=("Helvetica", 15, "bold"), bg="white",
                          fg="black", highlightbackground="white", text="Choose types of player by clicking the buttons.")
        choose_player.grid(row=1, column=0, pady=15, padx=10, columnspan=3)
        
        choose_player1 = tk.Label(self, width=20, height=1, font=("Helvetica", 15, "normal"), bg="white",
                          fg="black", highlightbackground="white", text="player 1:")
        choose_player1.grid(row=2, column=0, pady=1)
        
        choose_player2 = tk.Label(self, width=20, height=1, font=("Helvetica", 15, "normal"), bg="white",
                          fg="black", highlightbackground="white", text="player 2:")
        choose_player2.grid(row=2, column=2, pady=1)
        
        # create a button to start playing against an engine
        self.choosePlayer1 = tk.Button(self, text="Player", command=self.swap_player1,
                                        width=15, height=2, font=("Lucida", 12, "normal"),
                                        bg="white")
        self.choosePlayer1.grid(row=3, column=0, pady=1)
        
        # create a button to start playing against another player
        self.choosePlayer2 = tk.Button(self, text="Player", command=self.swap_player2,
                                       width=15, height=2, font=("Lucida", 12, "normal"),
                                       bg="white")
        self.choosePlayer2.grid(row=3, column=2, pady=1)
        
        # create a button to start go to gamePage
        self.startGame = tk.Button(self, text="Start play!", command=self.to_GamePage,
                                   width=40, height=2, font=("Lucida", 12, "normal"),
                                    bg="white")
        self.startGame.grid(row=4, column=0, pady=80, columnspan=3)
    
    def swap_player1(self):
        match self.player1Type:
            case PlayerType.Player:
                self.choosePlayer1.configure(text="Computer")
                self.player1Type = PlayerType.Computer
            case PlayerType.Computer:
                self.choosePlayer1.configure(text="Player")
                self.player1Type = PlayerType.Player

    def swap_player2(self):
        match self.player2Type:
            case PlayerType.Player:
                self.choosePlayer2.configure(text="Computer")
                self.player2Type = PlayerType.Computer
            case PlayerType.Computer:
                self.choosePlayer2.configure(text="Player")
                self.player2Type = PlayerType.Player
        
    def to_GamePage(self):
        self.controller.show_frame(GamePage)
        self.controller.frames[GamePage].recieve_playertypes(self.player1Type, self.player2Type)


class GamePage(tk.Frame):    
    def __init__(self, parent, controller):
        bg_color = "#504f5c"
        tk.Frame.__init__(self, parent)
        self.controller = controller
        self.configure(bg=bg_color)
        
        # recieve the coordinator
        self.coordinator = Coordinator()
        
        # create a list where all the canvas images will be stored
        self.pieces = []
        self.highlights = []
        self.legalmoves = []
        self.boardView = BoardView.White
        self.piece_size = int(BOARD_SIZE / 8)

        # keep track of playing or not
        self.playing = False
        
        # keep track whether we have selected a square
        self.selected = None # or is an int

        # create a dictionary for the piece images
        self.pieceImages = {}

        # load the chessboard on screen
        img = Image.open("pythonchess/images/EmptyChessBoard.png")
        img = crop_transparent_pgn(img).resize((BOARD_SIZE, BOARD_SIZE))
        self.boardBackground = ImageTk.PhotoImage(img)
        self.boardCanvas = tk.Canvas(self, bg="black", width=BOARD_SIZE, height=BOARD_SIZE, bd=0,
                                     highlightthickness=0, relief="ridge")
        self.boardCanvas.grid(rowspan=7, columnspan=4, row=2, column=0, padx=15, pady=2)
        self.boardCanvas.create_image(0, 0, image=self.boardBackground, anchor="nw")
        
        # load piece images
        self.loadImages()

        # bind the mouse click
        self.boardCanvas.bind("<Button-1>", self.selectAndMove)
        # return to homescreen
        self.homescreen = tk.Button(self, text="homescreen", command=self.to_StartPage,
                                    width=30, height=1, font=("Lucida", 12, "normal"))
        self.homescreen.grid(row=0, column=1, columnspan=1, pady=2)
        homescreen_text = tk.Label(self, width=40, height=1, font=("Helvetica", 15, "bold"), bg=bg_color,
                          fg="white", highlightbackground=bg_color, text="Return to homescreen:")
        homescreen_text.grid(row=0, column=0, pady=4)
        # player texts
        self.player1_text = tk.Label(self, width=40, height=1, font=("Helvetica", 13, "normal"), bg=bg_color,
                          fg="white", highlightbackground=bg_color, text="player 1:", justify='left', anchor="w")
        self.player1_text.grid(row=1, column=0, pady=0)
        self.player2_text = tk.Label(self, width=40, height=1, font=("Helvetica", 13, "normal"), bg=bg_color,
                          fg="white", highlightbackground=bg_color, text="player 2:", justify='left', anchor="w")
        self.player2_text.grid(row=9, column=0, pady=0)
        # create a button that resets the board
        self.reset = tk.Button(self, text="reset", command=self.resetPosition,
                               width=15, height=2, font=("Lucida", 12, "normal"))
        self.reset.grid(row=3, column=4, padx=1, pady=5)
        # create a button that empty's the board
        self.empty = tk.Button(self, text="empty board", command=self.emptyPosition,
                               width=15, height=2, font=("Lucida", 12, "normal"))
        self.empty.grid(row=4, column=4, padx=1, pady=5)
        # create a button for undo
        self.undoButton = tk.Button(self, text="undo last move", command=self.undo,
                                    width=15, height=2, font=("Lucida", 12, "normal"))
        self.undoButton.grid(row=5, column=4, padx=1, pady=10)
        # create a button for flipping the board
        self.flipButton = tk.Button(self, text="flip board", command=self.flipBoard,
                                    width=15, height=2, font=("Lucida", 12, "normal"))
        self.flipButton.grid(row=6, column=4, padx=1, pady=5)

        self.playButton = tk.Button(self, text="Play!", command=self.startPlay,
                                    width=15, height=2, font=("Lucida", 12, "normal"))
        self.playButton.grid(row=2, column=4)

        # to finish the initialization we empty the board
        self.emptyPosition()

    def loadImages(self):
        size = self.piece_size
        # load all piece images
        self.pieceImages['p'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackPawn.png").resize((size, size)))
        self.pieceImages['r'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackRook.png").resize((size, size)))
        self.pieceImages['b'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackBishop.png").resize((size, size)))
        self.pieceImages['n'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackKnight.png").resize((size, size)))
        self.pieceImages['k'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackKing.png").resize((size, size)))
        self.pieceImages['q'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackQueen.png").resize((size, size)))
        self.pieceImages['P'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhitePawn.png").resize((size, size)))
        self.pieceImages['R'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteRook.png").resize((size, size)))
        self.pieceImages['B'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteBishop.png").resize((size, size)))
        self.pieceImages['N'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteKnight.png").resize((size, size)))
        self.pieceImages['K'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteKing.png").resize((size, size)))
        self.pieceImages['Q'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteQueen.png").resize((size, size)))
        self.pieceImages[' '] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/EmptySquare.png").resize((size, size)))
        self.pieceImages["SelectedSquare"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/SelectedSquare.png").resize((size, size)))
        self.pieceImages["LegalMove"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/LegalMove.png").resize((size, size)))
        self.pieceImages["CaptureMove"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/Capture.png").resize((size, size)))
    
    def recieve_playertypes(self, player1: PlayerType, player2: PlayerType):
            match player1:
                case PlayerType.Player:
                    self.player1_text.config(text="player 1: human")
                case PlayerType.Computer:
                    self.player1_text.config(text="player 1: computer")
            match player2:
                case PlayerType.Player:
                    self.player2_text.config(text="player 2: human")
                case PlayerType.Computer:
                    self.player2_text.config(text="player 2: computer")
        
    
    def to_StartPage(self):
        self.controller.show_frame(StartPage)
    
    def startPlay(self):
        self.playing = not self.playing
        if not self.playing:
            self.playButton.configure(text="Playing...")
            self.update()
        else:
            self.playButton.configure(text="Play!")

    def board2VisualBoard(self, rank: int, file: int) -> (float, float):
        return (file - 1) * self.piece_size, (8 - rank) * self. piece_size

    def visualBoard2Board(self, x: float, y: float) -> (int, int):
        file = int(x / self.piece_size) + 1
        rank = 8 - int(y / self.piece_size)
        return rank, file

    def flipBoard(self):
        match self.boardView:
            case BoardView.White:
                self.boardView = BoardView.Black
            case BoardView.Black:
                self.boardView = BoardView.White
        self.update()

    def highlightSquare(self, index: int):
        # now we want to highlight this square by adding a highlight piece
        rank, file = index2rank_file(index)
        x, y = self.board2VisualBoard(rank, file)
        new_highlight = self.boardCanvas.create_image(x, y, image=self.pieceImages["SelectedSquare"], anchor="nw")
        self.highlights.append(new_highlight)
        # bring the pieces above the highlights
        for piece in self.pieces:
            self.boardCanvas.lift(piece)

    def highlightLegalMoves(self, indices: list[int], move_type: MoveType):
        # The indices must be a list with integers denoting the indices of the squares to highlight
        match move_type:
            case MoveType.NonCapture:
                img = self.pieceImages["LegalMove"]
            case MoveType.Capture:
                img = self.pieceImages["CaptureMove"]
        
        for i in indices:
            rank, file = index2rank_file(i)
            x, y = self.board2VisualBoard(rank, file)
            new_highlight = self.boardCanvas.create_image(x, y, image=img, anchor="nw")
            self.highlights.append(new_highlight)
            
        # Lift the pieces over the highlights
        for piece in self.pieces:
            self.boardCanvas.lift(piece)

    def selectAndMove(self, event):
        # this function will send to the backend what square was pressed
        rank, file = self.visualBoard2Board(event.x, event.y)
        index = rank_file2index(rank, file)
        # invert the index if we are having blacks perspective
        if self.boardView == BoardView.Black:
            index = 63 - index
        # check if the rank and file are valid (not choosing outside the board)
        if index < 0 or index > 63:
            return
        else:
            self.coordinator.recieve_click(index)
        self.update()
            
            
    def addPiece(self, piece: str, rank: int, file: int):
        x, y = self.board2VisualBoard(rank, file)
        new_piece = self.boardCanvas.create_image(x, y, image=self.pieceImages[piece], anchor="nw")
        self.pieces.append(new_piece)

    def loadPosition(self, position: str):
        if self.boardView == BoardView.Black:
            # reverse the string
            position = position[::-1]
        # input is a string of lenght 64
        for index, piece in enumerate(position):
            rank, file = index2rank_file(index)
            self.addPiece(piece, rank, file)
            
    def resetPosition(self):
        self.coordinator.reset_position()
        self.update()
    
    def emptyPosition(self):
        self.coordinator = Coordinator()
        self.update()
    
    def undo(self):
        pass
    
    def update(self):
        # clear pieces and highlights etc
        for square in self.legalmoves + self.highlights + self.pieces:
            self.boardCanvas.delete(square)
        self.pieces = []
        self.highlights = []
        self.legalmoves = []
        # get the select square
        for i in self.coordinator.get_select(): # only option are [] and [i], this is just a trick to use an alternative to Option<>
            # change index depending upon perspective
            if self.boardView == BoardView.Black:
                i = 63 - i
            legal_captures = self.coordinator.get_legal_captures(i)
            legal_non_captures = self.coordinator.get_legal_non_captures(i)
            if self.boardView == BoardView.Black:
                legal_captures = [63 - index for index in legal_captures]
                legal_captures = [63 - index for index in legal_captures]
            # highlight the select square
            self.highlightSquare(i)
            # highlight legal capture and non capture moves
            self.highlightLegalMoves(legal_captures, MoveType.Capture)
            self.highlightLegalMoves(legal_non_captures, MoveType.NonCapture)
        # load the position
        self.loadPosition(self.coordinator.board_to_string())



if __name__ == '__main__':
    test = Visual()
    # The main loop for the application
    test.mainloop()

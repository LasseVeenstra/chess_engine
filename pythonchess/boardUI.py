import tkinter as tk
from tkinter import ttk
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


class GamePage(ttk.Frame):    
    def __init__(self, parent, controller):
        tk.Frame.__init__(self, parent)
        self.controller = controller
        
        # recieve the coordinator
        self.coordinator = Coordinator()
        
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
        self.boardView = BoardView.White
        self.piece_size = int(BOARD_SIZE / 8)

        # keep track of playing or not
        self.playing = False
        
        # keep track whether we have selected a square
        self.selected = None # or is an int

        # create a dictionary for the piece images
        self.pieceImages = {}

        # load the chessboard on screen
        boardFrame = ttk.Frame(self)
        img = Image.open("pythonchess/images/EmptyChessBoard.png")
        img = crop_transparent_pgn(img).resize((BOARD_SIZE, BOARD_SIZE))
        self.boardBackground = ImageTk.PhotoImage(img)
        self.boardCanvas = tk.Canvas(boardFrame, bg="black", width=BOARD_SIZE, height=BOARD_SIZE, bd=0,
                                     highlightthickness=0, relief="ridge")
        self.boardCanvas.create_image(0, 0, image=self.boardBackground, anchor="nw")
        self.boardCanvas.grid(rowspan=1, columnspan=6, row=1, column=0, padx=15, pady=2)
        # player texts
        self.player1_text = ttk.Label(boardFrame, font=("Helvetica", 13, "normal"), text="player 1:", anchor="w")
        self.player1_text.grid(row=0, column=0, pady=0)
        self.player2_text = ttk.Label(boardFrame, font=("Helvetica", 13, "normal"), text="player 2:", anchor="w")
        self.player2_text.grid(row=2, column=0, pady=0)
        
        boardFrame.grid(row=1, column=0, columnspan=2)
        
        # load piece images
        self.loadImages()

        # bind the mouse click
        self.boardCanvas.bind("<ButtonPress-1>", self.on_select)
        # bind the dragging of the pieces
        self.boardCanvas.bind("<B1-Motion>", self.on_drag)
        self.boardCanvas.bind("<ButtonRelease-1>", self.on_drop)
        # return to homescreen
        homescreenFrame = ttk.Frame(self)
        self.homescreen = ttk.Button(homescreenFrame, text="homescreen", command=self.to_StartPage)
        self.homescreen.grid(row=0, column=1, columnspan=1, pady=2)
        homescreen_text = ttk.Label(homescreenFrame, font=("Helvetica", 15, "bold"), text="Return to homescreen:")
        homescreen_text.grid(row=0, column=0, pady=4)
        
        homescreenFrame.grid(row=0, column=0, pady=10)
        
        # create buttons
        buttonsFrame = ttk.Frame(self)
        
        # create a button that resets the board
        self.reset = ttk.Button(buttonsFrame, text="reset", command=self.resetPosition)
        self.reset.grid(row=0, column=0, padx=1, pady=5)
        # create a button that empty's the board
        self.empty = ttk.Button(buttonsFrame, text="empty board", command=self.emptyPosition)
        self.empty.grid(row=1, column=0, padx=1, pady=5)
        # create a button for undo
        self.undoButton = ttk.Button(buttonsFrame, text="undo last move", command=self.undo)
        self.undoButton.grid(row=2, column=0, padx=1, pady=5)
        # create a button for flipping the board
        self.flipButton = ttk.Button(buttonsFrame, text="flip board", command=self.flipBoard)
        self.flipButton.grid(row=3, column=0, padx=1, pady=5)
        self.playButton = ttk.Button(buttonsFrame, text="Play!", command=self.startPlay)
        self.playButton.grid(row=4, column=0, pady=5)
        
        buttonsFrame.grid(row=1, column=2, padx=10)
        
        
        
        self.fenUploadFrame = tk.Frame(self)
        self.fenEntry = ttk.Entry(self.fenUploadFrame)
        self.fenEntry.grid(row=0, column=0)
        fenSubmit = ttk.Button(self.fenUploadFrame, text="upload FEN", command=self.loadFEN)
        fenSubmit.grid(row=0, column=1)
        
        self.fenUploadFrame.grid(row=2, column=0, padx=5, pady=30)
        
        self.testButton = ttk.Button(self, text="test", command=self.test_positions)
        self.testButton.grid(row=2, column=1)

        # to finish the initialization we empty the board
        self.emptyPosition()
        self.update()

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

    def lift_pieces(self):
        # bring the pieces above the highlights
        piece_keys = ["p", "r", "b", "n", "k", "q", "P", "R", "B", "N", "K", "Q"]
        for piece in [self.image_ids.get(key) for key in piece_keys]:
            self.boardCanvas.lift(piece)
    
    def highlightSquare(self, index: int):
        # now we want to highlight this square by adding a highlight piece
        rank, file = index2rank_file(index)
        self.addPiece("SelectedSquare", rank, file, "highlights")

    def highlightLegalMoves(self, indices: list[int], move_type: MoveType):
        # The indices must be a list with integers denoting the indices of the squares to highlight
        match move_type:
            case MoveType.NonCapture:
                key = "legalmoves"
                img_str = "LegalMove"
            case MoveType.Capture:
                key = "capturemoves"
                img_str = "CaptureMove"
        
        for i in indices:
            rank, file = index2rank_file(i)
            self.addPiece(img_str, rank, file, key)
        self.lift_pieces()

    def send_input2coordinator(self, index: int):
        # invert the index if we are having blacks perspective
        if self.boardView == BoardView.Black:
            index = 63 - index
        # check if the rank and file are valid (not choosing outside the board)
        if index < 0 or index > 63:
            return
        else:
            self.coordinator.recieve_click(index)
    
    def on_select(self, event):
        # this function will send to the backend what square was pressed
        rank, file = self.visualBoard2Board(event.x, event.y)
        index = rank_file2index(rank, file)
        self.send_input2coordinator(index)
        # update the piece that is selected
        self.selected, = self.boardCanvas.find_closest(event.x, event.y)
        self.boardCanvas.lift(self.selected)
        self.selected_index = index
        self.update()
    
    def on_drag(self, event):
        self.boardCanvas.itemconfig(self.selected, anchor="center")
        self.boardCanvas.coords(self.selected, event.x, event.y)
    
    def on_drop(self, event):
        if self.selected is None:
            return
        self.boardCanvas.itemconfig(self.selected, anchor="nw")
        rank ,file = self.visualBoard2Board(event.x, event.y)
        x, y = self.board2VisualBoard(rank, file)
        index = rank_file2index(rank, file)
        # only if we moved the piece do we want to try to move it
        if index != self.selected_index:
            self.boardCanvas.coords(self.selected, x, y)
            self.send_input2coordinator(index)
        
        self.update()
            
    def addPiece(self, piece: str, rank: int, file: int, key: str):
        x, y = self.board2VisualBoard(rank, file)
        # get al hidden pieces
        hidden_pieces = self.boardCanvas.find_withtag("hiding")
        # get available pieces of correct type
        available_pieces = list(set(hidden_pieces) & set(self.image_ids[key]))
        # no available pieces
        if len(available_pieces) == 0:
            new_piece = self.boardCanvas.create_image(x, y, image=self.pieceImages[piece], anchor="nw")
            self.image_ids[key].append(new_piece)
        else:
            new_piece = available_pieces[0]
            self.boardCanvas.coords(new_piece, x, y)
            self.boardCanvas.dtag(new_piece, "hiding")
            self.boardCanvas.itemconfig(new_piece, state="normal")

    def loadPosition(self, position: str):
        if self.boardView == BoardView.Black:
            # reverse the string
            position = position[::-1]
        # input is a string of lenght 64
        for index, piece in enumerate(position):
            rank, file = index2rank_file(index)
            self.addPiece(piece, rank, file, piece)
            
    def resetPosition(self):
        self.coordinator.reset_position()
        self.update()
    
    def emptyPosition(self):
        self.coordinator = Coordinator()
        self.update()
    
    def undo(self):
        self.coordinator.undo()
        self.update()
        
    def test_positions(self):
        self.coordinator.chessboard.test_position_depth()
        self.update()
    
    def update(self):
        # clear pieces and highlights etc
        for key in self.image_ids:
            for square in self.image_ids[key]:
                self.boardCanvas.itemconfig(square, state="hidden", tags=("hiding"))
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
    
    def loadFEN(self):
        # now load the FEN
        FEN = self.fenEntry.get()
        self.coordinator.loadFEN(FEN)
        self.update()



if __name__ == '__main__':
    test = Visual()
    # The main loop for the application
    test.mainloop()

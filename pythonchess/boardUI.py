import tkinter as tk
from PIL import Image, ImageTk
from board_coordinator import *

def index2rank_file(i: int):
    return (8-(i/8), (i%8)+1)

def rank_file2index(rank: int, file:int):
    return ((8-rank)*8) + file - 1


class Visual(tk.Tk):
    def __init__(self, coordinator: Coordinator, *args, **kwargs):
        tk.Tk.__init__(self, *args, **kwargs)
        # recieve the coordinator
        self.coordinator = coordinator
        
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
        # create a list where all the canvas images will be stored
        self.pieces = []
        self.highlights = []
        self.legalmoves = []
        self.boardView = "White"
        
        # store the position we get from the backend, upper case will be white and lower
        # case be black.
        self.position = [] # list of lenght 64 of characters containing all the pieces.
        for _ in range(64):
            self.position.append(' ')
        # true denotes white to move and false denotes black to move
        self.toMove = True

        # initialize both players as players, this can later be changed to computer
        self.player1Type = "Player"
        self.player2Type = "Player"

        # keep track whether we want the computer to make moves
        self.playing = False

        # keep track whether we have selected a square
        self.selected = None # or is an int

        # create a dictionary for the piece images
        self.pieceImages = {}

        # load the chessboard on screen
        self.boardBackground = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/EmptyChessBoard.png"))
        self.boardCanvas = tk.Canvas(self, bg="black", width=520, height=520)
        self.boardCanvas.grid(rowspan=1, columnspan=4, row=1, column=1, padx=10, pady=5)
        self.boardCanvas.create_image(263, 263, image=self.boardBackground)

        # load all piece images
        self.pieceImages['p'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackPawn.png").resize((50, 50)))
        self.pieceImages['r'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackRook.png").resize((50, 50)))
        self.pieceImages['b'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackBishop.png").resize((50, 50)))
        self.pieceImages['n'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackKnight.png").resize((50, 50)))
        self.pieceImages['k'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackKing.png").resize((50, 50)))
        self.pieceImages['q'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/BlackQueen.png").resize((50, 50)))
        self.pieceImages['P'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhitePawn.png").resize((50, 50)))
        self.pieceImages['R'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteRook.png").resize((50, 50)))
        self.pieceImages['B'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteBishop.png").resize((50, 50)))
        self.pieceImages['N'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteKnight.png").resize((50, 50)))
        self.pieceImages['K'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteKing.png").resize((50, 50)))
        self.pieceImages['Q'] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/WhiteQueen.png").resize((50, 50)))
        self.pieceImages[' '] = ImageTk.PhotoImage(Image.open(
            "pythonchess/images/EmptySquare.png").resize((50, 50)))
        self.pieceImages["SelectedSquare"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/SelectedSquare.png").resize((64, 64)))
        self.pieceImages["LegalMove"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/LegalMove.png").resize((64, 64)))
        self.pieceImages["CaptureMove"] = ImageTk.PhotoImage(
            Image.open("pythonchess/images/Capture.png").resize((64, 64)))

        # bind the mouse click
        self.boardCanvas.bind("<Button-1>", self.selectAndMove)
        # create a button that loads the board
        self.reset = tk.Button(self, text="reset", command=self.resetPosition,
                               width=15, height=2, font=("Lucida", 12, "normal"))
        self.reset.grid(row=3, column=1, padx=1, pady=5)
        # create a button that empty's the board
        self.empty = tk.Button(self, text="empty board", command=self.emptyPosition,
                               width=15, height=2, font=("Lucida", 12, "normal"))
        self.empty.grid(row=3, column=2, padx=1, pady=5)
        # create a button for undo
        self.undoButton = tk.Button(self, text="undo last move", command=self.undo,
                                    width=15, height=2, font=("Lucida", 12, "normal"))
        self.undoButton.grid(row=3, column=3, padx=1, pady=10)
        # create a button for flipping the board
        self.undoButton = tk.Button(self, text="flip board", command=self.flipBoard,
                                    width=15, height=2, font=("Lucida", 12, "normal"))
        self.undoButton.grid(row=3, column=4, padx=1, pady=5)

        # create a button to start playing against an engine
        self.choosePlayer1 = tk.Button(self, text="Player", command=self.player1,
                                        width=15, height=2, font=("Lucida", 12, "normal"))
        self.choosePlayer1.grid(row=0, column=1, pady=5)

        # create a button to start playing against another player
        self.choosePlayer2 = tk.Button(self, text="Player", command=self.player2,
                                       width=15, height=2, font=("Lucida", 12, "normal"))
        self.choosePlayer2.grid(row=2, column=1, pady=5)

        self.playButton = tk.Button(self, text="Play!", command=self.startPlay,
                                    width=15, height=2, font=("Lucida", 12, "normal"))
        self.playButton.grid(row=2, column=4)

        # to finish the initialization we empty the board
        self.emptyPosition()

    def startPlay(self):
        if not self.playing:
            self.playButton.configure(text="Playing...")
            self.playing = True
            self.updatePosition()
        else:
            self.playButton.configure(text="Play!")
            self.playing = False

    def player1(self):
        if self.player1Type == "Player":
            self.choosePlayer1.configure(text="Computer")
            self.player1Type = "Computer"
        else:
            self.choosePlayer1.configure(text="Player")
            self.player1Type = "Player"

    def player2(self):
        if self.player2Type == "Player":
            self.choosePlayer2.configure(text="Computer")
            self.player2Type = "Computer"
        else:
            self.choosePlayer2.configure(text="Player")
            self.player2Type = "Player"

    def board2VisualBoard(self, x, y):
        if self.boardView == "Black":
            x = 7 - x
            y = 7 - y
        return 64 * y + 39, 64 * x + 39

    def visualBoard2Board(self, x, y):
        file = round((x - 39) / 64)
        rank = round((y - 39) / 64)
        if self.boardView == "White":
            return rank, file
        else:
            return 7-rank, 7-file

    def flipBoard(self):
        if self.boardView == "White":
            self.boardView = "Black"
        else:
            self.boardView = "White"
        self.updatePosition()

    def highlightSquare(self, index: int):
        # now we want to highlight this square by adding a highlight piece
        rank, file = index2rank_file(index)
        x, y = self.board2VisualBoard(rank, file)
        new_highlight = self.boardCanvas.create_image(x, y, image=self.pieceImages["SelectedSquare"])
        self.highlights.append(new_highlight)
        # bring the pieces above the highlights
        for piece in self.pieces:
            self.boardCanvas.lift(piece)

    def highlightLegalMoves(self, indices: list):
        # The indices must be a list with integers denoting the indices of the squares to highlight
        for i in indices:
            rank, file = index2rank_file(i)
            x, y = self.board2VisualBoard(rank, file)
            new_highlight = self.boardCanvas.create_image(x, y, image=self.pieceImages["LegalMove"])
            self.highlights.append(new_highlight)
            
        # Lift the pieces over the highlights
        for piece in self.pieces:
            self.boardCanvas.lift(piece)

    def selectAndMove(self, event):
        # this function will send to the backend what square was pressed
        rank, file = self.visualBoard2Board(event.x, event.y)
        index = rank_file2index(rank, file)
        # check if the rank and file are valid (not choosing outside the board)
        if index < 0 or index > 63:
            return
        else:
            return index
            
    def addPiece(self, piece: str, rank: int, file: int):
        x, y = self.board2VisualBoard(rank, file)
        new_piece = self.boardCanvas.create_image(x, y, image=self.pieceImages[piece])
        self.pieces.append(new_piece)

    def loadPosition(self, position: str):
        # input is a string of lenght 64
        for index, piece in enumerate(position):
            rank, file = index2rank_file(index)
            self.addPiece(piece, rank, file)
            
    def resetPosition(self):
        self.coordinator.reset_position()
    
    def emptyPosition(self):
        pass
    
    def undo(self):
        pass



if __name__ == '__main__':
    test = Visual()
    # The main loop for the application
    test.mainloop()

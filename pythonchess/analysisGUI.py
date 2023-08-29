import tkinter as tk
from tkinter import ttk
from PIL import Image, ImageTk
from enum import Enum, auto
import numpy as np
import RustEngine as rst
from startscreenGUI import BOARD_SIZE, BoardView, crop_transparent_pgn

class AnalysisPage(ttk.Frame):
    def __init__(self, parent, controller):
        tk.Frame.__init__(self, parent)
        self.controller = controller
        
        # recieve the coordinator
        self.coordinator = rst.Coordinator.new_human_vs_human()
        
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
        self.player1_text.grid(row=2, column=0, pady=0)
        self.player2_text = ttk.Label(boardFrame, font=("Helvetica", 13, "normal"), text="player 2:", anchor="w")
        self.player2_text.grid(row=0, column=0, pady=0)
        
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
        # create a button for next computer move
        self.NextCompButton = ttk.Button(buttonsFrame, text="next computer move", command=self.nextComputerMove)
        self.NextCompButton.grid(row=4, column=0, pady=5)
        # start a computer vs computer run
        self.NextCompButton = ttk.Button(buttonsFrame, text="run computer vs computer", command=self.computervscomputerRun)
        self.NextCompButton.grid(row=5, column=0, pady=5)
        
        
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
        self.update_board()
    
    def recieve_playertypes(self, player1: PlayerType, player2: PlayerType):
            match player1:
                case PlayerType.Player:
                    self.player1_text.config(text="player 1: human")
                    self.coordinator.set_player1("human")
                case PlayerType.Computer:
                    self.player1_text.config(text="player 1: computer")
                    self.coordinator.set_player1("random")
            match player2:
                case PlayerType.Player:
                    self.player2_text.config(text="player 2: human")
                    self.coordinator.set_player2("human")
                case PlayerType.Computer:
                    self.player2_text.config(text="player 2: computer")
                    self.coordinator.set_player2("random")
        
    
    def to_StartPage(self):
        self.controller.show_frame(StartPage)
    
    def nextComputerMove(self):
        self.coordinator.next_computer_move()
        self.update_board()
        
    def computervscomputerRun(self):
        for _ in range(100):
            self.nextComputerMove()
            #sleep(0.5)

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
        self.update_board()

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
            self.coordinator.input_select(index)
    
    def on_select(self, event):
        # this function will send to the backend what square was pressed
        rank, file = self.visualBoard2Board(event.x, event.y)
        index = rank_file2index(rank, file)
        self.send_input2coordinator(index)
        # update the piece that is selected
        self.selected, = self.boardCanvas.find_closest(event.x, event.y)
        self.boardCanvas.lift(self.selected)
        self.selected_index = index
        self.update_board()
    
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
        
        self.update_board()
            
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
        self.update_board()
    
    def emptyPosition(self):
        self.coordinator.empty_position()
        self.update_board()
    
    def undo(self):
        self.coordinator.undo()
        self.update_board()
        
    def test_positions(self):
        self.coordinator.test_positions()
        self.update_board()
    
    def update_board(self):
        # clear pieces and highlights etc
        for key in self.image_ids:
            for square in self.image_ids[key]:
                self.boardCanvas.itemconfig(square, state="hidden", tags=("hiding"))
        # get the select square
        selected_square = self.coordinator.get_selected()
        if selected_square != -1:
            # change index depending upon perspective
            if self.boardView == BoardView.Black:
                selected_square = 63 - selected_square
            legal_captures = self.coordinator.get_legal_captures(selected_square)
            legal_non_captures = self.coordinator.get_legal_non_captures(selected_square)
            if self.boardView == BoardView.Black:
                legal_captures = [63 - index for index in legal_captures]
                legal_captures = [63 - index for index in legal_captures]
            # highlight the select square
            self.highlightSquare(selected_square)
            # highlight legal capture and non capture moves
            self.highlightLegalMoves(legal_captures, MoveType.Capture)
            self.highlightLegalMoves(legal_non_captures, MoveType.NonCapture)
        # load the position
        self.loadPosition(self.coordinator.to_string())
        self.update_idletasks()
    
    def loadFEN(self):
        # now load the FEN
        FEN = self.fenEntry.get()
        self.coordinator.load_fen(FEN)
        self.update_board()
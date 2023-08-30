import tkinter as tk
from tkinter import ttk
from PIL import Image, ImageTk
from enum import Enum, auto
import numpy as np
import RustEngine as rst
from startscreenGUI import BOARD_SIZE, BoardView, crop_transparent_pgn
from board_frame import *


class ChessboardForAnalysis(ChessboardCanvas):
    def __init__(self, parent):
        super().__init__(parent)
        
        # keep track whether we have selected a square
        self.selected = None # or is an int when selected
        self.selected_index = None
        
        # bind mouse events
        self.bind_events()
    
    def bind_events(self):
        self.bind("<ButtonPress-1>", self.on_select)
        self.bind("<B1-Motion>", self.on_drag)
        self.bind("<ButtonRelease-1>", self.on_drop)
        
    # when we click on the board
    def on_select(self, event):
        # this function will send to the backend what square was pressed
        rank, file = self.visual_board2board(event.x, event.y)
        index = rank_file2index(rank, file)
        self.send_input2coordinator(index)
        # update the piece that is selected
        self.selected, = self.find_closest(event.x, event.y)
        # stop if we try to select the background
        if self.selected == self.background_tag:
            self.selected = None
            return
        self.lift(self.selected)
        self.selected_index = index
        self.update_board()

    # when we drag over the board
    def on_drag(self, event):
        if self.selected is None:
            return
        self.itemconfig(self.selected, anchor="center")
        self.coords(self.selected, event.x, event.y)
    
    # when we release again on the board
    def on_drop(self, event):
        if self.selected is None:
            return
        self.itemconfig(self.selected, anchor="nw")
        rank ,file = self.visual_board2board(event.x, event.y)
        x, y = self.board2visual_board(rank, file)
        index = rank_file2index(rank, file)
        # only if we moved the piece do we want to try to move it
        if index != self.selected_index:
            self.coords(self.selected, x, y)
            self.send_input2coordinator(index)
        self.update_board()
  
    def highlightSquare(self, index: int):
        # now we want to highlight this square by adding a highlight piece
        rank, file = index2rank_file(index)
        self.add_piece("selected", rank, file, "highlights")

    def highlightLegalMoves(self, indices: list[int], move_type: MoveType):
        # The indices must be a list with integers denoting the indices of the squares to highlight
        match move_type:
            case MoveType.NonCapture:
                key = "legalmoves"
                img_str = "legal"
            case MoveType.Capture:
                key = "capturemoves"
                img_str = "capture"
        
        for i in indices:
            rank, file = index2rank_file(i)
            self.add_piece(img_str, rank, file, key)
        self.lift_pieces()


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
    
    def to_StartPage(self):
        self.controller.show_frame(StartPage)
    
    def nextComputerMove(self):
        self.coordinator.next_computer_move()
        self.update_board()
        
    def computervscomputerRun(self):
        for _ in range(100):
            self.nextComputerMove()
            #sleep(0.5)
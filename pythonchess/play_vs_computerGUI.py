import tkinter as tk
from tkinter import ttk
import board_frame
from time import sleep
import threading
from board_frame import rank_file2index, index2rank_file, MoveType, BoardView

class Chessboard_for_PlayervsAI(board_frame.ChessboardCanvas):
    def __init__(self, parent, play_as: str):
        super().__init__(parent)
        self.play_as = play_as
        # initialize the two players as random agents
        if play_as == "player1":
            self.chessboard_coordinator.set_player1("human")
            self.chessboard_coordinator.set_player2("basic")
        elif play_as == "player2":
            self.chessboard_coordinator.set_player1("basic")
            self.chessboard_coordinator.set_player2("human")
            
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
        self.selected_rank_file_from = (rank, file)
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
        index = rank_file2index(rank, file)
        # always temporarely place the piece back to where it came from
        rank_from, file_from = self.selected_rank_file_from
        x_from, y_from = self.board2visual_board(rank_from, file_from)
        self.coords(self.selected, x_from, y_from)
        # only if we moved the piece do we want to try to move it
        if index != self.selected_index:
            self.send_input2coordinator(index)
        self.update_board()
        
    def send_input2coordinator(self, index: int):
        # invert the index if we are having blacks perspective
        if self.board_view == BoardView.Black:
            index = 63 - index
        # check if the rank and file are valid (not choosing outside the board)
        if index < 0 or index > 63:
            return
        else:
            self.chessboard_coordinator.input_select(index)
  
    def highlightSquare(self, index: int):
        # now we want to highlight this square by adding a highlight piece
        self.add_piece("selected", index, "highlights")

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
            self.add_piece(img_str, i, key)
        self.lift_pieces()
    
    def next_move(self):
        if (self.play_as == "player1" and self.chessboard_coordinator.get_to_move() == "player2") or (self.play_as == "player2" and self.chessboard_coordinator.get_to_move() == "player1"):
            self.chessboard_coordinator.next_computer_move()
            self.update_board()


class PlayervsAIPage(ttk.Frame):
    def __init__(self, parent, controller):
        ttk.Frame.__init__(self, parent)
        self.controller = controller
        
        # main chessboard
        self.chessboard = Chessboard_for_PlayervsAI(self, play_as="player1")
        self.chessboard.grid(row=0, column=0, padx=10, pady=10)
        
        # make all the buttons
        self.make_buttons()
        self.buttons_frame.grid(row=0, column=1)
    
    # set the as who we are playing
    def set_player_color(self, play_as: str):
        self.chessboard = Chessboard_for_PlayervsAI(self, play_as)
    
    def create_fen_upload(self, parent):
        self.fenUploadFrame = tk.Frame(parent)
        self.entry_str = tk.StringVar()
        fenEntry = ttk.Entry(self.fenUploadFrame, textvariable=self.entry_str)
        fenEntry.grid(row=0, column=0)
        fenSubmit = ttk.Button(self.fenUploadFrame, text="upload FEN", command=lambda: self.chessboard.load_fen(self.entry_str.get()))
        fenSubmit.grid(row=0, column=1)
    
    # make all the buttons
    def make_buttons(self):
        self.buttons_frame = ttk.Frame(self)
        self.start_stop_button = ttk.Button(self.buttons_frame, text="start", command=self.start_stop_play)
        self.start_stop_button.grid(row=0, column=0)
        # undo button
        ttk.Button(self.buttons_frame, text="undo", command=self.chessboard.undo).grid(row=1, column=0)
        ttk.Button(self.buttons_frame, text="move", command=self.chessboard.next_move).grid(row=2, column=0)
        # reset button
        ttk.Button(self.buttons_frame, text="reset", command=self.chessboard.reset_position).grid(row=3, column=0)
        self.create_fen_upload(self.buttons_frame)
        self.fenUploadFrame.grid(row=3, column=0)

    
    def playing_thread(self):
        while True:
            if self.pause_thread:
                break
            self.chessboard.next_move()
            self.update_idletasks()
    
    def start_stop_play(self):
        if self.start_stop_button["text"] == "start":
            self.start_stop_button["text"] = "stop"
            self.pause_thread = False
            self.play_thread = threading.Thread(target=self.playing_thread)
            self.play_thread.start()
        else:
            self.start_stop_button["text"] = "start"
            self.pause_thread = True
            
        
        